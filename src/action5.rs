
// mod error;
// mod identity;
// mod writer;

use std::marker::PhantomData;

// struct Map<'a, A, ActA, F> {
//     act_a: ActA,
//     func: F,
//     _act_a_type: PhantomData<&'a A>,
// }

// struct First<'b, B, ActA, ActB> {
//     act_a: ActA,
//     act_b: ActB,
//     _act_b_type: PhantomData<&'b B>,
// }
// struct Second<'a, A, ActA, ActB> {
//     act_a: ActA,
//     act_b: ActB,
//     _act_a_type: PhantomData<&'a A>,
// }
// struct Sequence<ActA, ActB>(ActA, ActB);
// struct Combine<'a, 'b, A, B, ActA, ActB, F> {
//     act_a: ActA,
//     act_b: ActB,
//     func: F,
//     _act_a_type: PhantomData<&'a A>,
//     _act_b_type: PhantomData<&'b B>,
// }

pub trait Monoid: Default {
    fn op(&self, other: &Self) -> Self;
}

impl Monoid for u32 {
    fn op(&self, other: &Self) -> Self {
        self + other
    }
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<A>(pub A);
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildId<S>(pub S);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildWriter<S, W>(pub PhantomData<*const W>, pub S);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildError<S, E>(pub PhantomData<*const E>, pub S);

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

/// The identity monad.
impl<A> Ap<A> for () {
    type R = A;
    fn build(value: A) -> Self::R {
        value
    }
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

impl<A, S, W> Ap<A> for BuildWriter<S, W>
where
    S: Ap<(A, W)>,
    W: Default,
{
    type R = S::R;
    fn build(value: A) -> Self::R {
        S::build((value, W::default()))
    }
}

impl<A, S, E> Ap<A> for BuildError<S, E>
where
    S: Ap<Result<A, E>>,
{
    type R = S::R;
    fn build(value: A) -> Self::R {
        S::build(Ok(value))
    }
}


// trait Mappable<A>
// where
//     Self: Sized,
// {
//     /// Map a structure element wise using a function. This version takes
//     /// ownership of `self`, so you can only map something once.
//     ///
//     /// When the structure also implements `Sequential` or `Action`, `map`
//     /// should perform the action, then modify the result using the function.
//     fn map<'a, B, F>(self, func: F) -> Map<'a, A, Self, F>
//     where
//         F: FnMut(&A) -> B,
//         Map<'a, A, Self, F>: Mappable<B>,
//     {
//         Map {
//             act_a: self,
//             func: func,
//             _act_a_type: PhantomData,
//         }
//     }
// }

// trait Sequential<A>: Mappable<A> {
//     fn first<'b, B, ActB>(self, other: ActB) -> First<'b, B, Self, ActB>
//     where
//         ActB: Sequential<B>,
//         First<'b, B, Self, ActB>: Sequential<A>,
//     {
//         First {
//             act_a: self,
//             act_b: other,
//             _act_b_type: PhantomData,
//         }
//     }

//     fn second<'a, B, ActB>(self, other: ActB) -> Second<'a, A, Self, ActB>
//     where
//         ActB: Sequential<B>,
//         Second<'a, A, Self, ActB>: Sequential<B>,
//     {
//         Second {
//             act_a: self,
//             act_b: other,
//             _act_a_type: PhantomData,
//         }
//     }

//     fn sequence<B, ActB>(self, other: ActB) -> Sequence<Self, ActB>
//     where
//         ActB: Sequential<B>,
//         Sequence<Self, ActB>: Sequential<(A, B)>,
//     {
//         Sequence(self, other)
//     }

//     fn combine<'a, 'b, B, ActB, F, R>(
//         self,
//         other: ActB,
//         func: F,
//     ) -> Combine<'a, 'b, A, B, Self, ActB, F>
//     where
//         ActB: Sequential<B>,
//         F: FnMut(&A, &B) -> R,
//         Combine<'a, 'b, A, B, Self, ActB, F>: Sequential<R>,
//     {
//         Combine {
//             act_a: self,
//             act_b: other,
//             func: func,
//             _act_a_type: PhantomData,
//             _act_b_type: PhantomData,
//         }
//     }
// }

trait Action<A>
where
    Self: Sized,
{
    /// Execute an action, and use it's result to
    fn and_then<B, ActB, F>(self, func: F) -> AndThen<A, Self, ActB, F>
    where
        F: Fn(&A) -> ActB,
        ActB: Action<B>,
        AndThen<A, Self, ActB, F>: Action<B>,
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
    fn and_then_ctx<'ab, B, ActB, Ctx>(
        self,
        context: Ctx,
        func: fn(&Ctx, &A) -> ActB,
    ) -> AndThenCtx<A, Self, ActB, Ctx>
    where
        ActB: Action<B>,
        AndThenCtx<A, Self, ActB, Ctx>: Action<B>,
    {
        AndThenCtx {
            act_a: self,
            func: func,
            context: context,
            _act_b: PhantomData,
        }
    }
}

trait Run<S, A>: Action<A>
where
    S: Ap<A>,
{
    fn run(&self) -> S::R;
    fn run_qualified(self, scaffold: S) -> S::R {
        self.run()
    }
    fn literal(&self) -> Literal<S, A, S::R> {
        literal(self.run())
    }
}

/// `Pure` is an action which does nothing except return a value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pure<A> {
    value: A,
}
pub fn pure<A>(value: A) -> Pure<A> {
    Pure { value: value }
}
impl<A> Action<A> for Pure<A> {}
/// Pure is an element of every type of action, so we can implement a run
/// instance for any type scaffold.
impl<S, A> Run<S, A> for Pure<A>
where
    S: Ap<A>,
    A: Clone,
{
    fn run(&self) -> S::R {
        S::build(self.value.clone())
    }
}



pub struct Literal<S, A, T> {
    value: T,
    _scaffold: PhantomData<*const S>,
    _type: PhantomData<*const A>,
}
impl<S, A, T> Literal<S, A, T> {
    fn value(self) -> T {
        self.value
    }
}
fn literal<S, A, T> (value: T) -> Literal<S, A, T> {
    Literal {
        value: value,
        _scaffold: PhantomData,
        _type: PhantomData,
    }
}
impl<A, S, T> Action<A> for Literal<S, A, T> where S: Ap<A, R = T>{}
impl<A, S, T> Run<S, A> for Literal<S, A, T> 
where
    S: Ap<A, R = T>,
    T: Clone,
{
    fn run(&self) -> S::R {
        self.value.clone()
    }
}
impl<A, S, T, W> Literal<BuildWriter<S, W>, A, T> {
    fn unwrap (self) -> Literal<S, (A, W), T> {
        literal(self.value)
    }
}
impl<A, S, T, W> Literal<S, (A, W), T> {
    fn wrap (self) -> Literal<BuildWriter<S, W>, A, T> {
        literal(self.value)
    }
}



/// Bind two actions together using the result from the first action to modify
/// the second.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AndThen<A, ActA, ActB, F> {
    act_a: ActA,
    func: F,
    _act_a_type: PhantomData<*const A>,
    _act_b: PhantomData<*const ActB>,
}
impl<A, B, ActA, ActB, F> Action<B> for AndThen<A, ActA, ActB, F>
where
    ActA: Action<A>,
    ActB: Action<B>,
    F: Fn(&A) -> ActB,
{
}
impl<A, B, ActA, ActB, F> Run<(), B> for AndThen<A, ActA, ActB, F>
where
    ActA: Run<(), A>,
    ActB: Run<(), B>,
    F: Fn(&A) -> ActB,
{
    fn run(&self) -> <() as Ap<B>>::R {
        (self.func)(&self.act_a.run()).run()
    }
}
impl<A, B, ActA, ActB, S, W, F> Run<BuildWriter<S, W>, B> for AndThen<A, ActA, ActB, F>
where
    B: Clone,
    W: Monoid + Clone,
    S: Ap<(B, W)> + Ap<(A, W)>,
    ActA: Run<BuildWriter<S, W>, A> + Run<S, (A, W)> + Clone,
    ActB: Run<BuildWriter<S, W>, B> + Action<(B, W)>,
    F: Fn(&A) -> ActB + Clone,
    AndThenCtx<A, ActA, ActB, F>: Run<BuildWriter<S, W>, B>,
{
    fn run(&self) -> <BuildWriter<S, W> as Ap<B>>::R {
        self.act_a
            .clone()
            .and_then_ctx(self.func.clone(), |func, a| func(a))
            .run()
    }
}

#[derive(Clone)]
struct AndThenCtx<A, ActA, ActB, Ctx> {
    act_a: ActA,
    func: fn(&Ctx, &A) -> ActB,
    context: Ctx,
    _act_b: PhantomData<*const ActB>,
}
impl<A, B, ActA, ActB, Ctx> Action<B> for AndThenCtx<A, ActA, ActB, Ctx> {}
impl<A, B, ActA, ActB, Ctx> Run<(), B> for AndThenCtx<A, ActA, ActB, Ctx>
where
    ActA: Run<(), A>,
    ActB: Run<(), B>,
{
    fn run(&self) -> <() as Ap<B>>::R {
        (self.func)(&self.context, &self.act_a.run()).run()
    }
}
impl<A, B, ActA, ActB, S, W, Ctx> Run<BuildWriter<S, W>, B> for AndThenCtx<A, ActA, ActB, Ctx>
where
    B: Clone,
    W: Monoid + Clone,
    Ctx: Clone,
    S: Ap<(B, W)> + Ap<(A, W)>,
    ActA: Run<BuildWriter<S, W>, A>,
    ActB: Action<(B, W)>,
    AndThenCtx<(A, W), Literal<S, (A, W), <S as Ap<(A, W)>>::R>, AndThenCtx<(B, W), ActB, Writer<B, W>, W>, (for<'r, 's> fn(&'r Ctx, &'s A) -> ActB, Ctx)>: Run<S, (B, W)>,
{
    fn run(&self) -> <BuildWriter<S, W> as Ap<B>>::R {
        self.act_a.literal().unwrap().and_then_ctx(
                (self.func, self.context.clone()),
                |(func, context), (a, w1)| {
                    func(context, a)
                        .and_then_ctx(w1.clone(), |w1, (b, w2)| writer(b.clone(), w1.op(&w2)))
                },
            )
            .literal()
            .wrap()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct Writer<A, W> {
    value: A,
    log: W,
}
fn writer<A, W> (value: A, log: W) -> Writer<A, W> {
    Writer {
        value: value,
        log: log,
    }
}
impl<A, W> Action<A> for Writer<A, W> {}
impl<S, W, A> Run<BuildWriter<S, W>, A> for Writer<A, W>
where
    S: Ap<(A, W)>,
    A: Clone,
    W: Default + Clone,
{
    fn run(&self) -> S::R {
        S::build((self.value.clone(), self.log.clone()))
    }
}


// impl<'a, A, B, ActA, F> Mappable<B> for Map<'a, A, ActA, F>
// where
//     ActA: Mappable<A>,
//     F: Fn(&A) -> B,
// {
// }
// impl<'a, A, B, ActA, F> Sequential<B> for Map<'a, A, ActA, F>
// where
//     ActA: Sequential<A>,
//     F: Fn(&A) -> B,
// {
// }
// impl<'a, A, B, ActA, F> Action<B> for Map<'a, A, ActA, F>
// where
//     ActA: Action<A>,
//     F: Fn(&A) -> B,
// {
// }

// // impl<A> Mappable<A> for Pure<A> {}
// // impl<A> Sequential<A> for Pure<A> {}
// // impl<A> Action<A> for Pure<A> {}

// impl<'b, A, B, ActA, ActB> Mappable<A> for First<'b, B, ActA, ActB>
// where
//     ActA: Mappable<A>,
//     ActB: Mappable<B>,
// {
// }
// impl<'b, A, B, ActA, ActB> Sequential<A> for First<'b, B, ActA, ActB>
// where
//     ActA: Sequential<A>,
//     ActB: Sequential<B>,
// {
// }
// impl<'b, A, B, ActA, ActB> Action<A> for First<'b, B, ActA, ActB>
// where
//     ActA: Action<A>,
//     ActB: Action<B>,
// {
// }

// impl<'a, A, B, ActA, ActB> Mappable<B> for Second<'a, A, ActA, ActB>
// where
//     ActA: Mappable<A>,
//     ActB: Mappable<B>,
// {
// }
// impl<'a, A, B, ActA, ActB> Sequential<B> for Second<'a, A, ActA, ActB>
// where
//     ActA: Sequential<A>,
//     ActB: Sequential<B>,
// {
// }
// impl<'a, A, B, ActA, ActB> Action<B> for Second<'a, A, ActA, ActB>
// where
//     ActA: Action<A>,
//     ActB: Action<B>,
// {
// }

// impl<A, B, ActA, ActB> Mappable<(A, B)> for Sequence<ActA, ActB>
// where
//     ActA: Mappable<A>,
//     ActB: Mappable<B>,
// {
// }
// impl<A, B, ActA, ActB> Sequential<(A, B)> for Sequence<ActA, ActB>
// where
//     ActA: Sequential<A>,
//     ActB: Sequential<B>,
// {
// }
// impl<A, B, ActA, ActB> Action<(A, B)> for Sequence<ActA, ActB>
// where
//     ActA: Action<A>,
//     ActB: Action<B>,
// {
// }

// impl<'a, 'b, A, B, C, ActA, ActB, F> Mappable<C> for Combine<'a, 'b, A, B, ActA, ActB, F>
// where
//     ActA: Mappable<A>,
//     ActB: Mappable<B>,
//     F: Fn(&A, &B) -> C,
// {
// }
// impl<'a, 'b, A, B, C, ActA, ActB, F> Sequential<C> for Combine<'a, 'b, A, B, ActA, ActB, F>
// where
//     ActA: Sequential<A>,
//     ActB: Sequential<B>,
//     F: Fn(&A, &B) -> C,
// {
// }
// impl<'a, 'b, A, B, C, ActA, ActB, F> Action<C> for Combine<'a, 'b, A, B, ActA, ActB, F>
// where
//     ActA: Action<A>,
//     ActB: Action<B>,
//     F: Fn(&A, &B) -> C,
// {
// }

// impl<'a, 'ab, A, B, ActA, ActB, F> Mappable<B> for AndThen<'a, 'ab, A, ActA, ActB, F>
// where
//     ActA: Mappable<A>,
//     ActB: Mappable<B>,
//     F: Fn(&A) -> ActB,
// {
// }
// impl<'a, 'ab, A, B, ActA, ActB, F> Sequential<B> for AndThen<'a, 'ab, A, ActA, ActB, F>
// where
//     ActA: Sequential<A>,
//     ActB: Sequential<B>,
//     F: Fn(&A) -> ActB,
// {
// }

// impl<'ab, A, B, ActA, ActB, Ctx> Mappable<B> for AndThenCtx<'ab, A, ActA, ActB, Ctx>
// where
//     ActA: Mappable<A>,
//     ActB: Mappable<B>,
// {}
// impl<'ab, A, B, ActA, ActB, Ctx> Sequential<B> for AndThenCtx<'ab, A, ActA, ActB, Ctx>
// where
//     ActA: Sequential<A>,
//     ActB: Sequential<B>,
// {}

#[cfg(test)]
mod test {
    use super::*;
    use std::marker::PhantomData;

    fn build_two<A, B, S>(
        _scaffold: S,
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
        let err_type: PhantomData<*const ()> = PhantomData;
        let writer_type: PhantomData<*const ()> = PhantomData;
        let scaffold = BuildWriter(writer_type, BuildError(err_type, BuildId(())));

        let (result_a, result_b) = build_two(scaffold, 32, "Hello");

        assert_eq!(result_a, Id(Ok((32, ()))));
        assert_eq!(result_b, Id(Ok(("Hello", ()))));
    }

    #[test]
    fn same_struct_different_value_2() {
        let err_type: PhantomData<*const ()> = PhantomData;
        let writer_type: PhantomData<*const ()> = PhantomData;
        let scaffold = BuildId(BuildError(err_type, BuildWriter(writer_type, ())));

        let (result_a, result_b) = build_two(scaffold, 32, "Hello");

        assert_eq!(result_a, (Ok(Id(32)), ()));
        assert_eq!(result_b, (Ok(Id("Hello")), ()));
    }

    #[test]
    fn run_and_then_identity() {
        let action = pure(3).and_then(|x| pure(x + 2)).and_then(|y| pure(y + 3));
        let action2 = pure(5).and_then(|x| pure(x + 7)).and_then(|y| pure(y + 9));

        assert_eq!(action.run(), 8);
        assert_eq!(action2.run(), 21);

        let action3 = action.and_then(|_| action2.clone());

        assert_eq!(action3.run(), 21);
    }

    #[test]
    fn pure_is_polymorphic() {
        let writer_type: PhantomData<*const u32> = PhantomData;
        let writer_scaffold = BuildWriter(writer_type, ());

        assert_eq!(pure(5).run_qualified(writer_scaffold), (5, 0));
        assert_eq!(pure(5).run_qualified(()), 5);
    }

    fn and_then_is_polymorphic() {
        let writer_type: PhantomData<*const u32> = PhantomData;
        let writer_scaffold = BuildWriter(writer_type, ());

        let action = pure(3).and_then_ctx((), |_, x| pure(x + 2));

        assert_eq!(action.run_qualified(()), 5);
        // assert_eq!(action.run_qualified(writer_scaffold), (5, 0));
    }

    #[test]
    fn run_and_then_writer() {
        let writer_type: PhantomData<*const u32> = PhantomData;
        let writer_scaffold = BuildWriter(writer_type, ());

        let action = pure(3).and_then(|x| pure(x + 2)).and_then(|y| pure(y + 3));
        let action2 = pure(5).and_then(|x| pure(x + 7)).and_then(|y| pure(y + 9));

        assert_eq!(action.run_qualified(writer_scaffold), (8, 0));
        // assert_eq!(action2.run_qualified(writer_scaffold), (21, 0));

        let action3 = action.and_then(|_| action2.clone());

        // assert_eq!(action3.run_qualified(writer_scaffold), (21, 0));
    }
}