mod identity;

use std::marker::PhantomData;

pub struct FromFunc<'a, 'b, A, B, F> {
    func: F,
    _input_type: PhantomData<&'a A>,
    _output_type: PhantomData<&'b B>,
}
pub fn f<'a, 'b, A, B, F>(func: F) -> FromFunc<'a, 'b, A, B, F> {
    FromFunc {
        func: func,
        _input_type: PhantomData,
        _output_type: PhantomData,
    }
}

pub struct Id<'a, A>(PhantomData<&'a A>);
pub fn id<'a, A>() -> Id<'a, A> {
    Id(PhantomData)
}

pub struct AndThen<'a, 'b, ActA: ?Sized, ActB: ?Sized> {
    act_a: &'a ActA,
    act_b: &'b ActB,
}

pub struct Sequence<'a, 'b, 'ia, 'ib, ActA: Action + ?Sized, ActB: Action + ?Sized> {
    act_a: &'a ActA,
    act_b: &'b ActB,
    _act_a_input_type: PhantomData<&'ia ActA::Input>,
    _act_b_input_type: PhantomData<&'ib ActB::Input>,
}

pub struct First<'a, 'ia, 'ib, ActA: Action + ?Sized, B> {
    act_a: &'a ActA,
    _act_a_input_type: PhantomData<&'ia ActA::Input>,
    _passthrough_input_type: PhantomData<&'ib B>,
}

pub struct Second<'b, 'ia, 'ib, A, ActB: Action + ?Sized> {
    act_b: &'b ActB,
    _passthrough_input_type: PhantomData<&'ia A>,
    _act_a_input_type: PhantomData<&'ib ActB::Input>,
}

pub struct Tee<'a, 'b, ActA: ?Sized, ActB: ?Sized> {
    act_a: &'a ActA,
    act_b: &'b ActB,
}

pub trait Action {
    type Input;
    type Output;

    fn and_then<'a, 'b, 'i, ActB>(&'a self, other: &'b ActB) -> AndThen<'a, 'b, Self, ActB>
    where
        Self::Input: 'i,
        ActB: Action<Input = Self::Output>,
        AndThen<'a, 'b, Self, ActB>: Action<Input = Self::Input, Output = ActB::Output>,
    {
        AndThen {
            act_a: self,
            act_b: other,
        }
    }

    fn sequence<'a, 'b, 'ia, 'ib, ActB>(
        &'a self,
        other: &'b ActB,
    ) -> Sequence<'a, 'b, 'ia, 'ib, Self, ActB>
    where
        ActB: Action,
        Self::Input: 'ia,
        ActB::Input: 'ib,
        Sequence<'a, 'b, 'ia, 'ib, Self, ActB>: Action<
            Input = (&'ia Self::Input, &'ib ActB::Input),
            Output = (Self::Output, ActB::Output),
        >,
    {
        Sequence {
            act_a: self,
            act_b: other,
            _act_a_input_type: PhantomData,
            _act_b_input_type: PhantomData,
        }
    }

    fn first<'a, 'b, 'ia, 'ib, B>(&'a self) -> First<'a, 'ia, 'ib, Self, B>
    where
        Self::Input: 'ia,
        B: 'ib + Clone,
        First<'a, 'ia, 'ib, Self, B>:
            Action<Input = (&'ia Self::Input, &'ib B), Output = (Self::Output, B)>,
    {
        First {
            act_a: self,
            _act_a_input_type: PhantomData,
            _passthrough_input_type: PhantomData,
        }
    }

    fn second<'a, 'b, 'ia, 'ib, A>(&'b self) -> Second<'b, 'ia, 'ib, A, Self>
    where
        A: 'ia + Clone,
        Self::Input: 'ib,
        Second<'b, 'ia, 'ib, A, Self>:
            Action<Input = (&'ia A, &'ib Self::Input), Output = (A, Self::Output)>,
    {
        Second {
            act_b: self,
            _passthrough_input_type: PhantomData,
            _act_a_input_type: PhantomData,
        }
    }

    fn tee<'a, 'b, 'i, ActB>(&'a self, other: &'b ActB) -> Tee<'a, 'b, Self, ActB>
    where
        Self::Input: 'i,
        ActB: Action<Input = Self::Input>,
        Tee<'a, 'b, Self, ActB>: Action<Input = Self::Input, Output = (Self::Output, ActB::Output)>,
    {
        Tee {
            act_a: self,
            act_b: other,
        }
    }
}

impl<'a, 'b, A, B, F> Action for FromFunc<'a, 'b, A, B, F>
where
    F: Fn(&A) -> B,
{
    type Input = A;
    type Output = B;
}

impl<'a, A> Action for Id<'a, A> {
    type Input = A;
    type Output = A;
}

impl<'a, 'b, 'i, ActA, ActB> Action for AndThen<'a, 'b, ActA, ActB>
where
    ActA: Action,
    ActB: Action<Input = ActA::Output>,
{
    type Input = ActA::Input;
    type Output = ActB::Output;
}

impl<'a, 'b, 'ia, 'ib, ActA, ActB> Action for Sequence<'a, 'b, 'ia, 'ib, ActA, ActB>
where
    ActA: Action,
    ActB: Action,
{
    type Input = (&'ia ActA::Input, &'ib ActB::Input);
    type Output = (ActA::Output, ActB::Output);
}

impl<'a, 'ia, 'ib, ActA, B> Action for First<'a, 'ia, 'ib, ActA, B>
where
    ActA: Action,
    B: Clone,
{
    type Input = (&'ia ActA::Input, &'ib B);
    type Output = (ActA::Output, B);
}

impl<'b, 'ia, 'ib, A, ActB> Action for Second<'b, 'ia, 'ib, A, ActB>
where
    ActB: Action,
    A: Clone,
{
    type Input = (&'ia A, &'ib ActB::Input);
    type Output = (A, ActB::Output);
}

impl<'a, 'b, ActA, ActB> Action for Tee<'a, 'b, ActA, ActB>
where
    ActA: Action,
    ActB: Action<Input = ActA::Input>,
{
    type Input = ActA::Input;
    type Output = (ActA::Output, ActB::Output);
}

enum Either<A, B> {
    Left(A),
    Right(B),
}
pub trait Error<E>: Action {
    fn run_error(&self, arg: &Self::Input) -> Either<E, Self::Output>;
}

impl<'a, 'b, A, B, F, E> Error<E> for FromFunc<'a, 'b, A, B, F>
where
    F: Fn(&A) -> B,
{
    fn run_error(&self, arg: &Self::Input) -> Either<E, Self::Output> {
        Either::Right((self.func)(arg))
    }
}

impl<'a, A, E> Error<E> for Id<'a, A>
where
    A: Clone,
{
    fn run_error(&self, arg: &Self::Input) -> Either<E, Self::Output> {
        Either::Right(arg.clone())
    }
}

impl<'a, 'b, 'i, ActA, ActB, E> Error<E> for AndThen<'a, 'b, ActA, ActB>
where
    ActA: Error<E>,
    ActB: Error<E, Input = ActA::Output>,
{
    fn run_error(&self, arg: &Self::Input) -> Either<E, Self::Output> {
        match self.act_a.run_error(arg) {
            Either::Right(middle) => self.act_b.run_error(&middle),
            Either::Left(error) => Either::Left(error),
        }
    }
}

impl<'a, 'b, 'ia, 'ib, ActA, ActB, E> Error<E> for Sequence<'a, 'b, 'ia, 'ib, ActA, ActB>
where
    ActA: Error<E>,
    ActB: Error<E>,
{
    fn run_error(&self, (arg_a, arg_b): &Self::Input) -> Either<E, Self::Output> {
        match self.act_a.run_error(arg_a) {
            Either::Left(error) => Either::Left(error),
            Either::Right(first) => match self.act_b.run_error(arg_b) {
                Either::Left(error) => Either::Left(error),
                Either::Right(second) => Either::Right((first, second)),
            },
        }
    }
}

impl<'a, 'ia, 'ib, ActA, B, E> Error<E> for First<'a, 'ia, 'ib, ActA, B>
where
    ActA: Error<E>,
    B: Clone,
{
    fn run_error(&self, args: &Self::Input) -> Either<E, Self::Output> {
        match self.act_a.run_error(args.0) {
            Either::Left(error) => Either::Left(error),
            Either::Right(result) => Either::Right((result, args.1.clone())),
        }
    }
}

impl<'b, 'ia, 'ib, A, ActB, E> Error<E> for Second<'b, 'ia, 'ib, A, ActB>
where
    ActB: Error<E>,
    A: Clone,
{
    fn run_error(&self, args: &Self::Input) -> Either<E, Self::Output> {
        match self.act_b.run_error(args.1) {
            Either::Left(error) => Either::Left(error),
            Either::Right(result) => Either::Right((args.0.clone(), result)),
        }
    }
}

impl<'a, 'b, ActA, ActB, E> Error<E> for Tee<'a, 'b, ActA, ActB>
where
    ActA: Error<E>,
    ActB: Error<E, Input = ActA::Input>,
{
    fn run_error(&self, arg: &Self::Input) -> Either<E, Self::Output> {
        match self.act_a.run_error(arg) {
            Either::Left(error) => Either::Left(error),
            Either::Right(first) => match self.act_b.run_error(arg) {
                Either::Left(error) => Either::Left(error),
                Either::Right(second) => Either::Right((first, second)),
            },
        }
    }
}
