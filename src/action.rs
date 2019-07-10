
mod error;
mod identity;
mod writer;

use std::marker::PhantomData;

fn id<T>(arg: T) -> T {
    arg
}

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