

mod error;
mod identity;
mod writer;

use std::marker::PhantomData;

struct Map<'a, A, ActA, F> {
    act_a: ActA,
    func: F,
    _act_a_type: PhantomData<&'a A>,
}

struct First<'b, B, ActA, ActB> {
    act_a: ActA,
    act_b: ActB,
    _act_b_type: PhantomData<&'b B>,
}
struct Second<'a, A, ActA, ActB> {
    act_a: ActA,
    act_b: ActB,
    _act_a_type: PhantomData<&'a A>,
}
struct Sequence<ActA, ActB>(ActA, ActB);
struct Combine<'a, 'b, A, B, ActA, ActB, F> {
    act_a: ActA,
    act_b: ActB,
    func: F,
    _act_a_type: PhantomData<&'a A>,
    _act_b_type: PhantomData<&'b B>,
}

struct AndThen<'a, 'ab, A, ActA, ActB, F> {
    act_a: ActA,
    func: F,
    _act_a_type: PhantomData<&'a A>,
    _act_b: PhantomData<&'ab ActB>,
}

struct AndThenCtx<'ab, A, ActA, ActB, Ctx> {
    act_a: ActA,
    func: fn(&Ctx, &A) -> ActB,
    context: Ctx,
    _act_b: PhantomData<&'ab ActB>,
}


/// BuildWriter, add this to the Writer file when appropriate.
pub struct BuildWriter<'w, S, W>(pub PhantomData<&'w W>, pub S);

impl<'w, A, S, W> TypeBuilder<A> for BuildWriter<'w, S, W>
where
    S: TypeBuilder<A>,
    (S::Output, W): BaseAction<A>,
    W: Default,
{
    type Output = (S::Output, W);
    fn build(value: A) -> Self::Output {
        (S::build(value), W::default())
    }
}



/// The typebuilder trait is implemented for type "scaffolds". Type scaffolds
/// are zero size markers which represent an incomplete type. When we run the
/// build function, we complete the type and produce a basic version of it. For
/// example, A single typeBuilder could create values of type `Result<A, E>` and
/// `Result<B, E>`
///
/// The build function is essentially the same as `pure` from `BaseAction`. The
/// difference is that 
pub trait TypeBuilder<A> {
    type Output: BaseAction<A>;
    fn build(value: A) -> Self::Output;
}

impl<A> TypeBuilder<A> for () {
    type Output = identity::Id<A>;
    fn build(value: A) -> Self::Output {
        identity::Id(value)
    }
}

trait Run<S, A>: Action<A>
where
    S: TypeBuilder<A>,
{
    fn run(&self) -> S::Output;
}

trait Mappable<A>
where
    Self: Sized,
{
    /// Map a structure element wise using a function. This version takes
    /// ownership of `self`, so you can only map something once.
    ///
    /// When the structure also implements `Sequential` or `Action`, `map`
    /// should perform the action, then modify the result using the function.
    fn map<'a, B, F>(self, func: F) -> Map<'a, A, Self, F>
    where
        F: FnMut(&A) -> B,
        Map<'a, A, Self, F>: Mappable<B>,
    {
        Map {
            act_a: self,
            func: func,
            _act_a_type: PhantomData,
        }
    }
}

/// The trait for concrete `Action` types. A base action is one that we can
/// create directly. using any kind of value. Other actions of the same return
/// type can be created by applying functions such as `map`, `sequence` and
/// `and_then`.
trait BaseAction<A>: Action<A> {
    fn pure(value: A) -> Self;
}

trait Sequential<A>: Mappable<A> {
    fn first<'b, B, ActB>(self, other: ActB) -> First<'b, B, Self, ActB>
    where
        ActB: Sequential<B>,
        First<'b, B, Self, ActB>: Sequential<A>,
    {
        First {
            act_a: self,
            act_b: other,
            _act_b_type: PhantomData,
        }
    }

    fn second<'a, B, ActB>(self, other: ActB) -> Second<'a, A, Self, ActB>
    where
        ActB: Sequential<B>,
        Second<'a, A, Self, ActB>: Sequential<B>,
    {
        Second {
            act_a: self,
            act_b: other,
            _act_a_type: PhantomData,
        }
    }

    fn sequence<B, ActB>(self, other: ActB) -> Sequence<Self, ActB>
    where
        ActB: Sequential<B>,
        Sequence<Self, ActB>: Sequential<(A, B)>,
    {
        Sequence(self, other)
    }

    fn combine<'a, 'b, B, ActB, F, R>(
        self,
        other: ActB,
        func: F,
    ) -> Combine<'a, 'b, A, B, Self, ActB, F>
    where
        ActB: Sequential<B>,
        F: FnMut(&A, &B) -> R,
        Combine<'a, 'b, A, B, Self, ActB, F>: Sequential<R>,
    {
        Combine {
            act_a: self,
            act_b: other,
            func: func,
            _act_a_type: PhantomData,
            _act_b_type: PhantomData,
        }
    }
}

trait Action<A>: Sequential<A> {
    /// Execute an action, and use it's result to 
    fn and_then<'a, 'ab, B, ActB, F>(self, func: F) -> AndThen<'a, 'ab, A, Self, ActB, F>
    where
        F: Fn(&A) -> ActB,
        ActB: Action<B>,
        AndThen<'a, 'ab, A, Self, ActB, F>: Action<B>,
    {
        AndThen {
            act_a: self,
            func: func,
            _act_a_type: PhantomData,
            _act_b: PhantomData,
        }
    }

    /// `and_then_ctx` has the same functionality to `and_then`, however instead
    /// of using the `Fn` trait for the closuer, we use a function pointer and a
    /// context object. This allows for ot's use where we wouldn't be able to
    /// know the type of a closure ahead of time.
    ///
    /// With the addition of `fn_traits` we may see this function dropped, as we
    /// will be able to refer to closures by type.
    fn and_then_ctx<'ab, B, ActB, Ctx>(self, context: Ctx, func: fn(&Ctx, &A) -> ActB) -> AndThenCtx<'ab, A, Self, ActB, Ctx>
    where
        ActB: Action<B>,
        AndThenCtx<'ab, A, Self, ActB, Ctx>: Action<B> 
    {
        AndThenCtx {
            act_a: self,
            func: func,
            context: context,
            _act_b: PhantomData,
        }
    }
}

impl<'a, A, B, ActA, F> Mappable<B> for Map<'a, A, ActA, F>
where
    ActA: Mappable<A>,
    F: Fn(&A) -> B,
{
}
impl<'a, A, B, ActA, F> Sequential<B> for Map<'a, A, ActA, F>
where
    ActA: Sequential<A>,
    F: Fn(&A) -> B,
{
}
impl<'a, A, B, ActA, F> Action<B> for Map<'a, A, ActA, F>
where
    ActA: Action<A>,
    F: Fn(&A) -> B,
{
}

// impl<A> Mappable<A> for Pure<A> {}
// impl<A> Sequential<A> for Pure<A> {}
// impl<A> Action<A> for Pure<A> {}

impl<'b, A, B, ActA, ActB> Mappable<A> for First<'b, B, ActA, ActB>
where
    ActA: Mappable<A>,
    ActB: Mappable<B>,
{
}
impl<'b, A, B, ActA, ActB> Sequential<A> for First<'b, B, ActA, ActB>
where
    ActA: Sequential<A>,
    ActB: Sequential<B>,
{
}
impl<'b, A, B, ActA, ActB> Action<A> for First<'b, B, ActA, ActB>
where
    ActA: Action<A>,
    ActB: Action<B>,
{
}

impl<'a, A, B, ActA, ActB> Mappable<B> for Second<'a, A, ActA, ActB>
where
    ActA: Mappable<A>,
    ActB: Mappable<B>,
{
}
impl<'a, A, B, ActA, ActB> Sequential<B> for Second<'a, A, ActA, ActB>
where
    ActA: Sequential<A>,
    ActB: Sequential<B>,
{
}
impl<'a, A, B, ActA, ActB> Action<B> for Second<'a, A, ActA, ActB>
where
    ActA: Action<A>,
    ActB: Action<B>,
{
}

impl<A, B, ActA, ActB> Mappable<(A, B)> for Sequence<ActA, ActB>
where
    ActA: Mappable<A>,
    ActB: Mappable<B>,
{
}
impl<A, B, ActA, ActB> Sequential<(A, B)> for Sequence<ActA, ActB>
where
    ActA: Sequential<A>,
    ActB: Sequential<B>,
{
}
impl<A, B, ActA, ActB> Action<(A, B)> for Sequence<ActA, ActB>
where
    ActA: Action<A>,
    ActB: Action<B>,
{
}

impl<'a, 'b, A, B, C, ActA, ActB, F> Mappable<C> for Combine<'a, 'b, A, B, ActA, ActB, F>
where
    ActA: Mappable<A>,
    ActB: Mappable<B>,
    F: Fn(&A, &B) -> C,
{
}
impl<'a, 'b, A, B, C, ActA, ActB, F> Sequential<C> for Combine<'a, 'b, A, B, ActA, ActB, F>
where
    ActA: Sequential<A>,
    ActB: Sequential<B>,
    F: Fn(&A, &B) -> C,
{
}
impl<'a, 'b, A, B, C, ActA, ActB, F> Action<C> for Combine<'a, 'b, A, B, ActA, ActB, F>
where
    ActA: Action<A>,
    ActB: Action<B>,
    F: Fn(&A, &B) -> C,
{
}

impl<'a, 'ab, A, B, ActA, ActB, F> Mappable<B> for AndThen<'a, 'ab, A, ActA, ActB, F>
where
    ActA: Mappable<A>,
    ActB: Mappable<B>,
    F: Fn(&A) -> ActB,
{
}
impl<'a, 'ab, A, B, ActA, ActB, F> Sequential<B> for AndThen<'a, 'ab, A, ActA, ActB, F>
where
    ActA: Sequential<A>,
    ActB: Sequential<B>,
    F: Fn(&A) -> ActB,
{
}
impl<'a, 'ab, A, B, ActA, ActB, F> Action<B> for AndThen<'a, 'ab, A, ActA, ActB, F>
where
    ActA: Action<A>,
    ActB: Action<B>,
    F: Fn(&A) -> ActB,
{
}

impl<'ab, A, B, ActA, ActB, Ctx> Mappable<B> for AndThenCtx<'ab, A, ActA, ActB, Ctx>
where
    ActA: Mappable<A>,
    ActB: Mappable<B>,
{}
impl<'ab, A, B, ActA, ActB, Ctx> Sequential<B> for AndThenCtx<'ab, A, ActA, ActB, Ctx>
where
    ActA: Sequential<A>,
    ActB: Sequential<B>,
{}
impl<'ab, A, B, ActA, ActB, Ctx> Action<B> for AndThenCtx<'ab, A, ActA, ActB, Ctx>
where
    ActA: Action<A>,
    ActB: Action<B>,
{}


#[cfg(test)]
mod test {
    use super::{BuildWriter, TypeBuilder};
    use super::error::BuildError;
    use super::identity::{BuildId, Id};
    use std::marker::PhantomData;

    fn build_two<A, B, S>(
        scaffold: S,
        value_a: A,
        value_b: B,
    ) -> (<S as TypeBuilder<A>>::Output, <S as TypeBuilder<B>>::Output)
    where
        S: TypeBuilder<A> + TypeBuilder<B>,
    {
        (S::build(value_a), S::build(value_b))
    }

    #[test]
    fn same_struct_different_value() {
        let err_type: PhantomData<&()> = PhantomData;
        let writer_type: PhantomData<&()> = PhantomData;
        let scaffold = BuildWriter(writer_type, BuildError(err_type, BuildId(())));

        let (result_a, result_b) = build_two(scaffold, 32, "Hello");

        assert_eq!(result_a, (Ok(Id(32)), ()));
        assert_eq!(result_b, (Ok(Id("Hello")), ()));
    }

    #[test]
    fn same_struct_different_value_2() {
        let err_type: PhantomData<&()> = PhantomData;
        let writer_type: PhantomData<&()> = PhantomData;
        let scaffold = BuildError(err_type, BuildWriter(writer_type, BuildId(())));

        let (result_a, result_b) = build_two(scaffold, 32, "Hello");

        assert_eq!(result_a, Ok((Id(32), ())));
        assert_eq!(result_b, Ok((Id("Hello"), ())));
    }
}