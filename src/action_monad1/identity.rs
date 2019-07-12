use super::{Action, AndThen, Combine, First, Map, Mappable, BaseAction, Second, Sequence, Sequential};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<A>(A);

trait Identity<A>: Action<A> {
    fn unwrap_identity(&self) -> Id<A>;
}

impl<A> Id<A> {
    fn run(self) -> A {
        self.0
    }
}

impl<A> Mappable<A> for Id<A> {}
impl<A> Sequential<A> for Id<A> {}
impl<A> Action<A> for Id<A> {}

impl<A> BaseAction<A> for Id<A> {
    fn pure(value: A) -> Self {
        Id(value)
    }
}

impl<A> Identity<A> for Id<A>
where
    A: Clone,
{
    fn unwrap_identity(&self) -> Id<A> {
        self.clone()
    }
}


impl<'a, A, B, ActA, F> Identity<B> for Map<'a, A, ActA, F>
where
    ActA: Identity<A>,
    F: Fn(&A) -> B,
{
    fn unwrap_identity(&self) -> Id<B> {
        Id((self.func)(&self.act_a.unwrap_identity().0))
    }
}
impl<'b, A, B, ActA, ActB> Identity<A> for First<'b, B, ActA, ActB>
where
    ActA: Identity<A>,
    ActB: Identity<B>,
{
    fn unwrap_identity(&self) -> Id<A> {
        let result = self.act_a.unwrap_identity();
        self.act_b.unwrap_identity();
        result
    }
}
impl<'a, A, B, ActA, ActB> Identity<B> for Second<'a, A, ActA, ActB>
where
    ActA: Identity<A>,
    ActB: Identity<B>,
{
    fn unwrap_identity(&self) -> Id<B> {
        self.act_a.unwrap_identity();
        self.act_b.unwrap_identity()
    }
}
impl<A, B, ActA, ActB> Identity<(A, B)> for Sequence<ActA, ActB>
where
    ActA: Identity<A>,
    ActB: Identity<B>,
{
    fn unwrap_identity(&self) -> Id<(A, B)> {
        Id((
            self.0.unwrap_identity().run(),
            self.1.unwrap_identity().run(),
        ))
    }
}
impl<'a, 'b, A, B, C, ActA, ActB, F> Identity<C> for Combine<'a, 'b, A, B, ActA, ActB, F>
where
    ActA: Identity<A>,
    ActB: Identity<B>,
    F: Fn(&A, &B) -> C,
{
    fn unwrap_identity(&self) -> Id<C> {
        let a = self.act_a.unwrap_identity().run();
        let b = self.act_b.unwrap_identity().run();
        Id((self.func)(&a, &b))
    }
}

impl<'a, 'ab, A, B, ActA, ActB, F> Identity<B> for AndThen<'a, 'ab, A, ActA, ActB, F>
where
    ActA: Identity<A>,
    ActB: Identity<B>,
    F: Fn(&A) -> ActB,
{
    fn unwrap_identity(&self) -> Id<B> {
        let a = self.act_a.unwrap_identity().run();
        (self.func)(&a).unwrap_identity()
    }
}