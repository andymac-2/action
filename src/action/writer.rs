use std::marker::PhantomData;

use super::{Action, AndThen, Combine, First, Map, Mappable, BaseAction, Second, Sequence, Sequential};

struct Listen<A, W>(A, W);
struct Tell<W>(W);

struct WriterTrans<'r, A, Act, W> {
    action: Act,
    _result_type: PhantomData<&'r (A, W)>,
}

trait Monoid: Default {
    fn op(&self, other: &Self) -> Self;
}

trait Writer<A, W>: Action<A> {
    fn unwrap_writer(&self) -> (A, W);
    fn listen(&self) -> Listen<A, W> 
    where 
        Listen<A, W>: Writer<(A, W), W>,
    {
        let (result, log) = self.unwrap_writer();
        Listen(result, log)
    }
}

trait WriterT<A, Act, W>: Action<A>
where
    Act: Action<(A, W)>,
{
    fn unwrap_writer_t(&self) -> Act;
}

trait BaseWriter<A, W>: BaseAction<A>
where
    Self: Sized,
{
    fn writer(value: A, log: W) -> Self;
    fn pass<ActA, F>(action: ActA) -> Self
    where
        ActA: Writer<(A, F), W>,
        F: Fn(&W) -> W;
}

trait BaseWriterT<A, Act, W>: BaseAction<A>
where
    Act: BaseAction<(A, W)>,
{
    fn writer_t(value: A, log: W) -> Act {
        Act::pure((value, log))
    }
    fn pass_t<ActA, ActInner, F>(action: ActA) -> Act
    where
        ActInner: BaseAction<((A, F), W)>,
        ActA: WriterT<(A, F), ActInner, W>,
        F: Fn(&W) -> W,
    {
        action.unwrap_writer_t()
    }
}

impl<W> Tell<W> {
    fn tell(log: W) -> Self {
        Tell(log)
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
    A: Default,
{
    fn writer(value: A, log: W) -> Self {
        (value, log)
    }
    fn pass<ActA, F>(action: ActA) -> Self
    where
        ActA: Writer<(A, F), W>,
        F: Fn(&W) -> W,
    {
        let ((result, func), log) = action.unwrap_writer();
        (result, func(&log))
    }
}

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
impl<'r, A, Act, W> Sequential<A> for WriterTrans<'r, A, Act, W> where W: Monoid, {}
impl<'r, A, Act, W> Action<A> for WriterTrans<'r, A, Act, W> where W: Monoid, {}


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
