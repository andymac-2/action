use std::marker::PhantomData;

use super::{
    Action, AndThen, BaseAction, Combine, First, Map, Mappable, Second, Sequence, Sequential,
};

struct WriterTrans<'r, A, Act, W> {
    action: Act,
    _result_type: PhantomData<&'r (A, W)>,
}

trait Monoid: Default {
    fn op(&self, other: &Self) -> Self;
}

trait Writer<A, W>: Action<A> {
    fn unwrap_writer(&self) -> (A, W);
}
trait BaseWriter<A, W>: BaseAction<A>
where
    Self: Sized,
{
    fn writer(value: A, log: W) -> Self;
    fn pass<ActA, F>(action: ActA) -> Self
    where
        ActA: Writer<(A, F), W>,
        F: Fn(&W) -> W,
    {
        let ((value, func), log) = action.unwrap_writer();
        Self::writer(value, func(&log))
    }
}
trait BaseWriterTell<W>: BaseWriter<(), W> {
    fn tell(log: W) -> Self {
        Self::writer((), log)
    }
}
trait BaseWriterListen<A, W>: BaseWriter<(A, W), W>
where
    W: Clone,
{
    fn listen<ActA>(action: ActA) -> Self
    where
        ActA: Writer<A, W>,
    {
        let (value, log) = action.unwrap_writer();
        Self::writer((value, log.clone()), log)
    }
}

trait WriterT<A, Act, W>: Action<A>
where
    Act: Action<(A, W)>,
{
    fn unwrap_writer_t(&self) -> Act;
}
trait BaseWriterT<A, Act, W>: BaseAction<A>
where
    Act: BaseAction<(A, W)>,
{
    fn writer_t(action: Act) -> Self;
}
trait BaseWriterTellT<Act, W>: BaseWriterT<(), Act, W>
where
    Act: BaseAction<((), W)>,
{
    fn tell(log: W) -> Self {
        Self::writer_t(Act::pure(((), log)))
    }
}

impl<A, W> Mappable<A> for (A, W) {}
impl<A, W> Sequential<A> for (A, W) where W: Monoid {}
impl<A, W> Action<A> for (A, W) where W: Monoid {}

impl<A, W> BaseAction<A> for (A, W)
where
    W: Monoid,
{
    fn pure(value: A) -> Self {
        (value, W::default())
    }
}

impl<A, W> Writer<A, W> for (A, W)
where
    W: Monoid,
    Self: Clone,
{
    fn unwrap_writer(&self) -> (A, W) {
        self.clone()
    }
}
impl<A, W> BaseWriter<A, W> for (A, W)
where
    W: Monoid,
{
    fn writer(value: A, log: W) -> Self {
        (value, log)
    }
}
impl<W> BaseWriterTell<W> for ((), W) where W: Monoid {}
impl<A, W> BaseWriterListen<A, W> for ((A, W), W) where W: Monoid + Clone {}

impl<'a, A, B, ActA, W, F> Writer<B, W> for Map<'a, A, ActA, F>
where
    ActA: Writer<A, W>,
    F: Fn(&A) -> B,
{
    fn unwrap_writer(&self) -> (B, W) {
        let (result, log) = self.act_a.unwrap_writer();
        ((self.func)(&result), log)
    }
}
impl<'b, A, B, ActA, ActB, W> Writer<A, W> for First<'b, B, ActA, ActB>
where
    W: Monoid,
    ActA: Writer<A, W>,
    ActB: Writer<B, W>,
{
    fn unwrap_writer(&self) -> (A, W) {
        let (result, log_a) = self.act_a.unwrap_writer();
        let (_, log_b) = self.act_b.unwrap_writer();
        (result, log_a.op(&log_b))
    }
}
impl<'a, A, B, ActA, ActB, W> Writer<B, W> for Second<'a, A, ActA, ActB>
where
    W: Monoid,
    ActA: Writer<A, W>,
    ActB: Writer<B, W>,
{
    fn unwrap_writer(&self) -> (B, W) {
        let (_, log_a) = self.act_a.unwrap_writer();
        let (result, log_b) = self.act_b.unwrap_writer();
        (result, log_a.op(&log_b))
    }
}
impl<A, B, ActA, ActB, W> Writer<(A, B), W> for Sequence<ActA, ActB>
where
    W: Monoid,
    ActA: Writer<A, W>,
    ActB: Writer<B, W>,
{
    fn unwrap_writer(&self) -> ((A, B), W) {
        let (result_a, log_a) = self.0.unwrap_writer();
        let (result_b, log_b) = self.1.unwrap_writer();
        ((result_a, result_b), log_a.op(&log_b))
    }
}
impl<'a, 'b, A, B, C, ActA, ActB, F, W> Writer<C, W> for Combine<'a, 'b, A, B, ActA, ActB, F>
where
    W: Monoid,
    ActA: Writer<A, W>,
    ActB: Writer<B, W>,
    F: Fn(&A, &B) -> C,
{
    fn unwrap_writer(&self) -> (C, W) {
        let (result_a, log_a) = self.act_a.unwrap_writer();
        let (result_b, log_b) = self.act_b.unwrap_writer();
        ((self.func)(&result_a, &result_b), log_a.op(&log_b))
    }
}

impl<'a, 'ab, A, B, ActA, ActB, F, W> Writer<B, W> for AndThen<'a, 'ab, A, ActA, ActB, F>
where
    W: Monoid,
    ActA: Writer<A, W>,
    ActB: Writer<B, W>,
    F: Fn(&A) -> ActB,
{
    fn unwrap_writer(&self) -> (B, W) {
        let (result_a, log_a) = self.act_a.unwrap_writer();
        let (result_b, log_b) = (self.func)(&result_a).unwrap_writer();
        (result_b, log_a.op(&log_b))
    }
}

// WriterT Implementations
impl<'r, A, Act, W> Mappable<A> for WriterTrans<'r, A, Act, W> {}
impl<'r, A, Act, W> Sequential<A> for WriterTrans<'r, A, Act, W>
where
    W: Monoid,
    Act: Action<(A, W)>,
{
}
impl<'r, A, Act, W> Action<A> for WriterTrans<'r, A, Act, W>
where
    W: Monoid,
    Act: Action<(A, W)>,
{
}
impl<'r, A, Act, W> BaseAction<A> for WriterTrans<'r, A, Act, W>
where
    W: Monoid,
    Act: BaseAction<(A, W)>,
{
    fn pure(value: A) -> Self {
        WriterTrans {
            action: Act::pure((value, W::default())),
            _result_type: PhantomData,
        }
    }
}
impl<'r, A, Act, W> WriterT<A, Act, W> for WriterTrans<'r, A, Act, W>
where
    W: Monoid,
    Act: Action<(A, W)> + Clone,
{
    fn unwrap_writer_t(&self) -> Act {
        self.action.clone()
    }
}
impl<'r, A, Act, W> BaseWriterT<A, Act, W> for WriterTrans<'r, A, Act, W>
where
    W: Monoid,
    Act: BaseAction<(A, W)>,
{
    fn writer_t(action: Act) -> Self {
        WriterTrans {
            action: action,
            _result_type: PhantomData,
        }
    }
}


// impl<'a, A, B, Act, ActA, W, F> WriterT<B, Act, W> for Map<'a, A, ActA, F>
// where
//     ActA: WriterT<A, Act, W>,
//     F: Fn(&A) -> B,
// {
//     fn unwrap_writer_t(&self) -> (B, W) {
//         let (result, log) = self.act_a.unwrap_writer_t();
//         ((self.func)(&result), log)
//     }
// }
// impl<'b, A, B, ActA, ActB, W> WriterT<A, Act, W> for First<'b, B, ActA, ActB>
// where
//     W: Monoid,
//     ActA: WriterT<A, Act, W>,
//     ActB: WriterT<B, Act, W>,
// {
//     fn unwrap_writer(&self) -> (A, W) {
//         let (result, log_a) = self.act_a.unwrap_writer();
//         let (_, log_b) = self.act_b.unwrap_writer();
//         (result, log_a.op(&log_b))
//     }
// }
// impl<'a, A, B, ActA, ActB, W> WriterT<B, Act, W> for Second<'a, A, ActA, ActB>
// where
//     W: Monoid,
//     ActA: WriterT<A, Act, W>,
//     ActB: WriterT<B, Act, W>,
// {
//     fn unwrap_writer(&self) -> (B, W) {
//         let (_, log_a) = self.act_a.unwrap_writer();
//         let (result, log_b) = self.act_b.unwrap_writer();
//         (result, log_a.op(&log_b))
//     }
// }
// impl<A, B, ActA, ActB, W> Writer<(A, B), W> for Sequence<ActA, ActB>
// where
//     W: Monoid,
//     ActA: WriterT<A, Act, W>,
//     ActB: WriterT<B, Act, W>,
// {
//     fn unwrap_writer(&self) -> ((A, B), W) {
//         let (result_a, log_a) = self.0.unwrap_writer();
//         let (result_b, log_b) = self.1.unwrap_writer();
//         ((result_a, result_b), log_a.op(&log_b))
//     }
// }
// impl<'a, 'b, A, B, C, ActA, ActB, F, W> Writer<C, W> for Combine<'a, 'b, A, B, ActA, ActB, F>
// where
//     W: Monoid,
//     ActA: WriterT<A, Act, W>,
//     ActB: WriterT<B, Act, W>,
//     F: Fn(&A, &B) -> C,
// {
//     fn unwrap_writer(&self) -> (C, W) {
//         let (result_a, log_a) = self.act_a.unwrap_writer();
//         let (result_b, log_b) = self.act_b.unwrap_writer();
//         ((self.func)(&result_a, &result_b), log_a.op(&log_b))
//     }
// }

// impl<'a, 'ab, A, B, ActA, ActB, F, W> WriterT<B, Act, W> for AndThen<'a, 'ab, A, ActA, ActB, F>
// where
//     W: Monoid,
//     ActA: WriterT<A, Act, W>,
//     ActB: Writer<B, W>,
//     F: Fn(&A) -> ActB,
// {
//     fn unwrap_writer(&self) -> (B, W) {
//         let (result_a, log_a) = self.act_a.unwrap_writer();
//         let (result_b, log_b) = (self.func)(&result_a).unwrap_writer();
//         (result_b, log_a.op(&log_b))
//     }
// }


// I have no idea on how to even format this type.
fn do_something<A: Functor<u32, u32>>(input: A) ->
    <<<<A as Functor<u32, u32>>::Output as Functor<u32, u32>>::Output as Functor<u32, u32>>::Output as Functor<u32, u32>>::Output
where
    A::Output : Functor<u32, u32>,
    <<A as Functor<u32, u32>>::Output as Functor<u32, u32>>::Output: Functor<u32, u32>,
    <<<A as Functor<u32, u32>>::Output as Functor<u32, u32>>::Output as Functor<u32, u32>>::Output: Functor<u32, u32>,
{
    input
        .fmap(|y| y + 3)
        .fmap(|y| y * 2)
        .fmap(|y| y - 10)
        .fmap(|y| y + 5)
}

fn cnst<A, B>(a: A) -> impl FnOnce(B) -> A {
    |_| a
}

trait Functor<A, B> {
    type Output;

    fn fmap<F>(self, f: F) -> Self::Output
    where
        F: FnOnce(A) -> B;
}