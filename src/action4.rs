use std::marker::PhantomData;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<A>(pub A);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildId<S>(pub S);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildWriter<'w, S, W>(pub PhantomData<&'w W>, pub S);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildError<'e, S, E>(pub PhantomData<&'e E>, pub S);

/// The typebuilder trait is implemented for type "scaffolds". Type scaffolds
/// are zero size markers which represent an incomplete type. When we run the
/// build function, we complete the type and produce a basic version of it. For
/// example, A single typeBuilder could create values of type `Result<A, E>` and
/// `Result<B, E>`
///
/// The build function is essentially the same as `pure` from `BaseAction`. The
/// difference is that 
pub trait Ap<A> {
    type R;
    fn build(value: A) -> Self::R;
}

impl<A, S> Ap<A> for BuildId<S> 
where
    S: Ap<Id<A>>,
{
    type R = S::R;
    fn build(value: A) -> Self::R {
        S::build(Id(value))
    }
}

impl<'w, A, S, W> Ap<A> for BuildWriter<'w, S, W> 
where
    S: Ap<(A, W)>,
    W: Default,
{
    type R = S::R;
    fn build(value: A) -> Self::R {
        S::build((value, W::default()))
    }
}

impl<'e, A, S, E> Ap<A> for BuildError<'e, S, E>
where
    S: Ap<Result<A, E>>,
{
    type R = S::R;
    fn build(value: A) -> Self::R {
        S::build(Ok(value))
    }
}

/// The trait for concrete `Action` types. A base action is one that we can
/// create directly. using any kind of value. Other actions of the same return
/// type can be created by applying functions such as `map`, `sequence` and
/// `and_then`.
trait BaseAction<A> {
    fn pure(value: A) -> Self;
}

trait Mappable<A>: Ap<A>
where
    Self: Sized,
{
    /// Map a structure element wise using a function. This version takes
    /// ownership of `self`, so you can only map something once.
    ///
    /// When the structure also implements `Sequential` or `Action`, `map`
    /// should perform the action, then modify the result using the function.
    fn map<B, F>(act: <Self as Ap<A>>::R, func: F) -> <Self as Ap<B>>::R
    where
        Self: Ap<B>,
        F: FnMut(&A) -> B
    {
        Self::map_ctx(act, func, |func_inner, a| {
            func_inner(a)
        })
    }

    fn map_ctx<B, Ctx>(act: <Self as Ap<A>>::R, context: Ctx, func: fn(&Ctx, &A) -> B) -> <Self as Ap<B>>::R
    where
        Self: Ap<B>;
}

trait Sequential<A>: Mappable<A> {
    fn first<B>(act_a: <Self as Ap<A>>::R, act_b: <Self as Ap<B>>::R) -> <Self as Ap<A>>::R
    where
        A: Clone,
        Self: Ap<B>,
    {
        Self::combine(act_a, act_b, |a: &A, _b: &B| a.clone())
    }

    fn second<B>(act_a: <Self as Ap<A>>::R, act_b: <Self as Ap<B>>::R) -> <Self as Ap<B>>::R
    where
        B: Clone,
        Self: Ap<B>,
    {
        Self::combine(act_a, act_b, |_a: &A, b: &B| b.clone())
    }

    fn sequence<B>(act_a: <Self as Ap<A>>::R, act_b: <Self as Ap<B>>::R) -> <Self as Ap<(A, B)>>::R
    where
        A: Clone,
        B: Clone,
        Self: Ap<B> + Ap<(A, B)>,
    {
        Self::combine(act_a, act_b, |a: &A, b: &B| (a.clone(), b.clone()))
    }

    fn combine<B, C, F>(act_a: <Self as Ap<A>>::R, act_b: <Self as Ap<B>>::R, func: F) -> <Self as Ap<C>>::R
    where
        Self: Ap<B> + Ap<C>,
        F: FnMut(&A, &B) -> C
    {
        Self::combine_ctx(act_a, act_b, func, |func_inner, a, b| {
            func_inner(a, b)
        })
    }

    fn combine_ctx<B, C, Ctx>(act_a: <Self as Ap<A>>::R, act_b: <Self as Ap<B>>::R, contxt: Ctx, func: fn(&Ctx, &A, &B) -> C) -> <Self as Ap<C>>::R
    where
        Self: Ap<B> + Ap<C>;
}

trait Action<A>: Sequential<A> {
    fn and_then<B, F>(act_a: <Self as Ap<A>>::R, func: F) -> <Self as Ap<B>>::R
    where
        Self: Ap<B>,
        F: Fn(&A) -> <Self as Ap<B>>::R
    {
        Self::and_then_ctx(act_a, func, |func_inner, a| {
            func_inner(a)
        })
    }

    /// `and_then_ctx` has the same functionality to `and_then`, however instead
    /// of using the `Fn` trait for the closuer, we use a function pointer and a
    /// context object. This allows for ot's use where we wouldn't be able to
    /// know the type of a closure ahead of time.
    ///
    /// With the addition of `fn_traits` we may see this function dropped, as we
    /// will be able to refer to closures by type.
    fn and_then_ctx<B, Ctx>(act_a: <Self as Ap<A>>::R, context: Ctx, func: fn(&Ctx, &A) -> <Self as Ap<B>>::R) -> <Self as Ap<B>>::R
    where
        Self: Ap<A> + Ap<B>;
}


/// The identity monad.
impl<A> Ap<A> for () {
    type R = A;
    fn build(value: A) -> Self::R {
        value
    }
}
impl<A> Mappable<A> for () 
where
{
    fn map_ctx<B, Ctx>(act: <Self as Ap<A>>::R, context: Ctx, func: fn(&Ctx, &A) -> B) -> <Self as Ap<B>>::R
    where
        Self: Ap<A, R = A> + Ap<B>
    {
        Self::build(func(&context, &act))
    }
}
impl<A> Sequential<A> for () 
where
    Self: Ap<A, R = A>,
{
    fn combine_ctx<B, C, Ctx>(act_a: <Self as Ap<A>>::R, act_b: <Self as Ap<B>>::R, context: Ctx, func: fn(&Ctx, &A, &B) -> C) -> <Self as Ap<C>>::R
    where
        Self: Ap<B> + Ap<C>
    {
        Self::and_then(act_a, |a| {
            <Self as Action<B>>::and_then(act_b, |b| {
                Self::build(func(&context, &a, &b))
            })
        })
    }
}
impl<A> Action<A> for () 
where 
    Self: Ap<A, R = A>,
{
    fn and_then_ctx<B, Ctx>(act_a: <Self as Ap<A>>::R, context: Ctx, func: fn(&Ctx, &A) -> <Self as Ap<B>>::R) -> <Self as Ap<B>>::R
    where
        Self: Ap<A, R = A> + Ap<B>
    {
        func(&context, &act_a)
    }
}


#[cfg(test)]
mod test {
    use super::{BuildId, Id, BuildError, BuildWriter, Ap};
    use std::marker::PhantomData;

    fn build_two<A, B, S>(
        scaffold: S,
        value_a: A,
        value_b: B,
    ) -> (<S as Ap<A>>::R, <S as Ap<B>>::R)
    where
        S: Ap<A> + Ap<B>,
    {
        (S::build(value_a), S::build(value_b))
    }

    #[test]
    fn same_struct_different_value() {
        let err_type: PhantomData<&()> = PhantomData;
        let writer_type: PhantomData<&()> = PhantomData;
        let scaffold = BuildWriter(writer_type, BuildError(err_type, BuildId(())));

        let (result_a, result_b) = build_two(scaffold, 32, "Hello");

        assert_eq!(result_a, Id(Ok((32, ()))));
        assert_eq!(result_b, Id(Ok(("Hello", ()))));
    }

    #[test]
    fn same_struct_different_value_2() {
        let err_type: PhantomData<&()> = PhantomData;
        let writer_type: PhantomData<&()> = PhantomData;
        let scaffold = BuildId(BuildError(err_type, BuildWriter(writer_type, ())));

        let (result_a, result_b) = build_two(scaffold, 32, "Hello");

        assert_eq!(result_a, (Ok(Id(32)), ()));
        assert_eq!(result_b, (Ok(Id("Hello")), ()));
    }
}