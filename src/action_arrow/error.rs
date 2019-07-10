use super::identity::{Identity, LiftIdentity};
use super::{Action, AndThen, First, FromFunc, Id, Pure, Second, Sequence, Tee};
use std::marker::PhantomData;

struct LiftError<A>(A);
struct IntoError<A>(A);
pub struct RunError<'e, Act, E> {
    action: Act,
    _error_type: PhantomData<&'e E>,
}

pub struct FuncError<'a, 'b, A, B, F> {
    func: F,
    _input_type: PhantomData<&'a A>,
    _output_type: PhantomData<&'b B>,
}
pub fn e<'a, 'b, A, B, F>(func: F) -> FuncError<'a, 'b, A, B, F> {
    FuncError {
        func: func,
        _input_type: PhantomData,
        _output_type: PhantomData,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Either<A, B> {
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

impl<'a, A, E> Error<E> for Pure<'a, A>
where
    A: Clone,
{
    fn run_error(&self, _arg: &Self::Input) -> Either<E, Self::Output> {
        Either::Right(self.0.clone())
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

impl<Act> Action for LiftIdentity<IntoError<Act>>
where
    Act: Action,
{
    type Input = Act::Input;
    type Output = Act::Output;
}
impl<A, E> Error<E> for LiftIdentity<IntoError<A>>
where
    A: Identity,
{
    fn run_error(&self, arg: &Self::Input) -> Either<E, Self::Output> {
        Either::Right((self.0).0.run(arg))
    }
}

impl<Act> Action for LiftError<IntoError<Act>>
where
    Act: Action,
{
    type Input = Act::Input;
    type Output = Act::Output;
}
impl<Act, E> Error<E> for LiftError<IntoError<Act>>
where
    Act: Error<E>,
{
    fn run_error(&self, arg: &Self::Input) -> Either<E, Act::Output> {
        (self.0).0.run_error(arg)
    }
}

impl<'a, 'b, A, B, F> Action for FuncError<'a, 'b, A, B, F> {
    type Input = A;
    type Output = B;
}
impl<'a, 'b, A, B, F, E> Error<E> for FuncError<'a, 'b, A, B, F>
where
    F: Fn(&A) -> Either<E, B>,
{
    fn run_error(&self, arg: &Self::Input) -> Either<E, Self::Output> {
        (self.func)(arg)
    }
}

pub trait ErrorT<E>: Action {
    fn strip_errors(&self) -> RunError<&Self, E> {
        RunError {
            action: self, 
            _error_type: PhantomData
        }
    }
}

impl<'e, Act, E> Action for RunError<'e, &Act, E> 
    where Act: Error<E>
{
    type Input = Act::Input;
    type Output = Either<E, Act::Output>;
}


impl<'e, Act, E> Identity for RunError<'e, &Act, E> 
    where Act: Error<E>
{
    fn run(&self, arg: &Self::Input) -> Self::Output {
        self.action.run_error(arg)
    }
}


#[cfg(test)]
mod test {
    use super::{e, Either, Error};
    use crate::action::{f, id, Action, Identity};

    #[test]
    fn errors_single() {
        let chain = e(|x: &u32| {
            if x == &0 {
                Either::Left(())
            } else {
                Either::Right(100 / x)
            }
        });

        assert_eq!(chain.run_error(&2), Either::Right(50));
        assert_eq!(chain.run_error(&4), Either::Right(25));
    }

    #[test]
    fn errors_chain() {
        let f1 = e(|x: &i32| Either::Right(x + 2));
        let f2 = e(|x: &i32| {
            if x == &0 {
                Either::Left(())
            } else {
                Either::Right(100 / x)
            }
        });
        let f3 = e(|x: &i32| Either::Right(x + 10));

        let first_two = f1.and_then(&f2);
        let chain = first_two.and_then(&f3);

        assert_eq!(chain.run_error(&2), Either::Right(35));
        assert_eq!(chain.run_error(&3), Either::Right(30));
        assert_eq!(chain.run_error(&-2), Either::Left(()));
    }

    #[test]
    fn mix_identity_with_error() {
        let f1 = f(|x: &i32| x + 2);
        let f2 = e(|x: &i32| {
            if x == &0 {
                Either::Left(())
            } else {
                Either::Right(100 / x)
            }
        });
        let f3 = f(|x: &i32| x + 10);

        let first_two = f1.and_then(&f2);
        let chain = first_two.and_then(&f3);

        assert_eq!(chain.run_error(&2), Either::Right(35));
        assert_eq!(chain.run_error(&3), Either::Right(30));
        assert_eq!(chain.run_error(&-2), Either::Left(()));

        // uncommenting the following line should give an error.
        // chain.run_identity(&2);
    }

    #[test]
    fn identity_or_error() {
        let f1 = f(|x: &i32| x + 2);
        let f2 = id();
        let f3 = f(|x: &i32| x + 10);

        let first_two = f1.and_then(&f2);
        let chain = first_two.and_then(&f3);

        assert_eq!(chain.run_error(&2), Either::<(), _>::Right(14));
        assert_eq!(chain.run(&3), 15);
    }
}