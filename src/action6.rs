
// mod error;
// mod identity;
// mod writer;

use std::marker::PhantomData;

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
pub struct Identity<A>(A);


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildId();

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildWriter<W>(pub PhantomData<*const W>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildError<E>(pub PhantomData<*const E>);

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

impl<A> Ap<A> for BuildId {
    type R = Identity<A>;
    fn build(value: A) -> Self::R {
        Identity(value)
    }
}

impl<A, W> Ap<A> for BuildWriter<W>
where
    W: Default,
{
    type R = (A, W);
    fn build(value: A) -> Self::R {
        (value, W::default())
    }
}

impl<A, E> Ap<A> for BuildError<E> {
    type R = Result<A, E>;
    fn build(value: A) -> Self::R {
        Ok(value)
    }
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
    fn map<B, F>(self, func: F) -> Map<A, Self, F>
    where
        F: FnMut(&A) -> B,
        Map<A, Self, F>: Mappable<B>,
    {
        Map {
            act_a: self,
            func: func,
            _act_a_type: PhantomData,
        }
    }
}

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
    /// Execute an action, and use it's result to execute the next action.
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
}

trait Run<S, A>: Action<A>
where
    S: Ap<A>,
{
    fn run(&self) -> S::R;
    fn run_qualified(&self, _scaffold: &S) -> S::R {
        self.run()
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
impl<A> Mappable<A> for Pure<A> {}
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


pub struct Literal<S, A>
where
    S: Ap<A>,
{
    value: S::R,
}
pub fn literal<S, A>(value: S::R) -> Literal<S, A>
where
    S: Ap<A>,
{
    Literal { value: value }
}
impl<S, A> Mappable<A> for Literal<S, A> where S: Ap<A> {}
impl<S, A> Action<A> for Literal<S, A> where S: Ap<A> {}
impl<S, A> Run<S, A> for Literal<S, A>
where
    S: Ap<A>,
    S::R: Clone,
{
    fn run(&self) -> S::R {
        self.value.clone()
    }
}


struct Map<A, ActA, F> {
    act_a: ActA,
    func: F,
    _act_a_type: PhantomData<*const A>,
}
impl<A, B, ActA, F> Mappable<B> for Map<A, ActA, F>
where
    ActA: Mappable<A>,
    F: Fn(&A) -> B,
{
}
impl<A, B, ActA, F> Action<B> for Map<A, ActA, F>
where
    ActA: Action<A>,
    F: Fn(&A) -> B,
{
}
impl<A, B, ActA, F> Run<(), B> for Map<A, ActA, F> 
where
    ActA: Run<(), A>,
    F: Fn(&A) -> B,
{
    fn run(&self) -> <() as Ap<B>>::R {
        (self.func)(&self.act_a.run())
    }
}
impl<A, B, ActA, F, W> Run<BuildWriter<W>, B> for Map<A, ActA, F> 
where
    W: Default,
    ActA: Run<BuildWriter<W>, A>,
    F: Fn(&A) -> B,
{
    fn run(&self) -> <BuildWriter<W> as Ap<B>>::R {
        let (result_a, log) = self.act_a.run();
        ((self.func)(&result_a), log)
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
impl<A, B, ActA, ActB, F> Mappable<B> for AndThen<A, ActA, ActB, F>
where
    ActA: Action<A>,
    ActB: Action<B>,
    F: Fn(&A) -> ActB,
{
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
impl<A, B, ActA, ActB, W, F> Run<BuildWriter<W>, B> for AndThen<A, ActA, ActB, F>
where
    W: Monoid,
    ActA: Run<BuildWriter<W>, A>,
    ActB: Run<BuildWriter<W>, B>,
    F: Fn(&A) -> ActB,
{
    fn run(&self) -> <BuildWriter<W> as Ap<B>>::R {
        let (result_a, log_a) = self.act_a.run();
        let (result_b, log_b) = (self.func)(&result_a).run();
        (result_b, log_a.op(&log_b))
    }
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Writer<A, W> {
    value: A,
    log: W,
}
pub fn writer<A, W>(value: A, log: W) -> Writer<A, W> {
    Writer {
        value: value,
        log: log,
    }
}
impl<A, W> Mappable<A> for Writer<A, W> {}
impl<A, W> Action<A> for Writer<A, W> {}
impl<W, A> Run<BuildWriter<W>, A> for Writer<A, W>
where
    A: Clone,
    W: Default + Clone,
{
    fn run(&self) -> <BuildWriter<W> as Ap<A>>::R {
        (self.value.clone(), self.log.clone())
    }
}


// impl<'a, A, B, ActA, F> Sequential<B> for Map<'a, A, ActA, F>
// where
//     ActA: Sequential<A>,
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
        let writer_type: PhantomData<*const ()> = PhantomData;
        let scaffold = BuildWriter(writer_type);

        let (result_a, result_b) = build_two(scaffold, 32, "Hello");

        assert_eq!(result_a, (32, ()));
        assert_eq!(result_b, ("Hello", ()));
    }

    #[test]
    fn same_struct_different_value_2() {
        let err_type: PhantomData<*const ()> = PhantomData;
        let scaffold = BuildError(err_type);

        let (result_a, result_b) = build_two(scaffold, 32, "Hello");

        assert_eq!(result_a, Ok(32));
        assert_eq!(result_b, Ok("Hello"));
    }

    #[test]
    fn run_and_then_identity() {
        let action = pure(3).and_then(|x| pure(x + 2)).and_then(|y| pure(y + 3));
        let action2 = pure(5).and_then(|x| pure(x + 7)).and_then(|y| pure(y + 9));

        assert_eq!(action.run_qualified(&()), 8);
        assert_eq!(action2.run_qualified(&()), 21);

        let action3 = action.and_then(|_| action2.clone());

        assert_eq!(action3.run_qualified(&()), 21);
    }

    #[test]
    fn pure_is_polymorphic() {
        let writer_type: PhantomData<*const ()> = PhantomData;
        let scaffold = BuildWriter(writer_type);

        assert_eq!(pure(5).run_qualified(&scaffold), (5, ()));
        assert_eq!(pure(5).run_qualified(&()), 5);
    }

    #[test]
    fn and_then_is_polymorphic() {
        let writer_type: PhantomData<*const u32> = PhantomData;
        let scaffold = BuildWriter(writer_type);

        let action = pure(3).and_then(|x| pure(x + 2));

        assert_eq!(action.run_qualified(&()), 5);
        assert_eq!(action.run_qualified(&scaffold), (5, 0));
    }

    #[test]
    fn run_and_then_writer() {
        let writer_type: PhantomData<*const u32> = PhantomData;
        let scaffold = BuildWriter(writer_type);

        let action = pure(3).and_then(|x| pure(x + 2)).and_then(|y| pure(y + 3));
        let action2 = pure(5).and_then(|x| pure(x + 7)).and_then(|y| pure(y + 9));

        assert_eq!(action.run_qualified(&scaffold), (8, 0));
        assert_eq!(action2.run_qualified(&scaffold), (21, 0));

        let action3 = action.and_then(|_| action2.clone());

        assert_eq!(action3.run_qualified(&scaffold), (21, 0));
    }

    #[test]
    fn basic_writer() {
        let writer_type: PhantomData<*const u32> = PhantomData;
        let scaffold = BuildWriter(writer_type);

        let action = pure(true)
            .and_then(|x| writer(!x, 3))
            .and_then(|y| writer(!y, 3));

        let action2 = writer(true, 10)
            .and_then(|x| writer(!x, 10))
            .and_then(|y| writer(!y, 10));

        assert_eq!(action.run_qualified(&scaffold), (true, 6));
        assert_eq!(action2.run_qualified(&scaffold), (true, 30));

        let action3 = action.and_then(|_| action2.clone());

        assert_eq!(action3.run_qualified(&scaffold), (true, 36));
    }
}