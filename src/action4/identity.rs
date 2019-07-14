use std::marker::PhantomData;

use super::{
    Action, AndThen, BaseAction, Combine, First, Map, Mappable, Run, Second, Sequence, Sequential, TypeBuilder,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BuildId<S>(pub S);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<A>(pub A);

impl<A, S> TypeBuilder<A> for BuildId<S> where
    S: TypeBuilder<Id<A>>,
    S::Output: BaseAction<A>,
{
    type Output = S::Output;
    fn build(value: A) -> Self::Output {
        S::build(Id(value))
    }
}

impl<A> Id<A> {
    fn unwrap(self) -> A {
        self.0
    }
}

impl<A, Base> Mappable<A> for Id<Base> {}
impl<A, Base> Sequential<A> for Id<Base> where Base: BaseAction<A> {}
impl<A, Base> Action<A> for Id<Base> where Base: BaseAction<A> {}

impl<A, Base> BaseAction<A> for Id<Base>
where
    Base: BaseAction<A>
{
    fn pure(value: A) -> Self {
        Id(Base::pure(value))
    }
}

impl<A> Run<BuildId, A> for Id<A>
where
    A: Clone,
{
    fn run(&self) -> Id<A> {
        self.clone()
    }
}

impl<'a, A, B, ActA, F> Run<BuildId, B> for Map<'a, A, ActA, F>
where
    ActA: Run<BuildId, A>,
    F: Fn(&A) -> B,
{
    fn run(&self) -> Id<B> {
        Id((self.func)(&self.act_a.run().0))
    }
}
impl<'b, A, B, ActA, ActB> Run<BuildId, A> for First<'b, B, ActA, ActB>
where
    ActA: Run<BuildId, A>,
    ActB: Run<BuildId, B>,
{
    fn run(&self) -> Id<A> {
        let result = self.act_a.run();
        self.act_b.run();
        result
    }
}
impl<'a, A, B, ActA, ActB> Run<BuildId, B> for Second<'a, A, ActA, ActB>
where
    ActA: Run<BuildId, A>,
    ActB: Run<BuildId, B>,
{
    fn run(&self) -> Id<B> {
        self.act_a.run();
        self.act_b.run()
    }
}
impl<A, B, ActA, ActB> Run<BuildId, (A, B)> for Sequence<ActA, ActB>
where
    ActA: Run<BuildId, A>,
    ActB: Run<BuildId, B>,
{
    fn run(&self) -> Id<(A, B)> {
        Id((self.0.run().0, self.1.run().0))
    }
}
impl<'a, 'b, A, B, C, ActA, ActB, F> Run<BuildId, C> for Combine<'a, 'b, A, B, ActA, ActB, F>
where
    ActA: Run<BuildId, A>,
    ActB: Run<BuildId, B>,
    F: Fn(&A, &B) -> C,
{
    fn run(&self) -> Id<C> {
        let a = self.act_a.run().0;
        let b = self.act_b.run().0;
        Id((self.func)(&a, &b))
    }
}

impl<'a, 'ab, A, B, ActA, ActB, F> Run<BuildId, B> for AndThen<'a, 'ab, A, ActA, ActB, F>
where
    ActA: Run<BuildId, A>,
    ActB: Run<BuildId, B>,
    F: Fn(&A) -> ActB,
{
    fn run(&self) -> Id<B> {
        let a = self.act_a.run().0;
        (self.func)(&a).run()
    }
}