use super::{Action, AndThen, Combine, First, Map, Mappable, BaseAction, Second, Sequence, Sequential};

trait Error<A, E>: Action<A> {
    fn unwrap_error(&self) -> Result<A, E>;
}

trait BaseError<A, E>: BaseAction<A> 
where
    Self: Sized,
{
    fn throw_error(err: &E) -> Self;
    fn catch_error<ActA, ActB, F>(action: &ActA, func: F) -> Self
    where
        ActA: Error<A, E>,
        ActB: Error<A, E>,
        F: Fn(&E) -> ActB,
    {
        match action.unwrap_error() {
            Ok(result_a) => Self::pure(result_a),
            Err(err_a) => match func(&err_a).unwrap_error() {
                Ok(result_b) => Self::pure(result_b),
                Err(err_b) => Self::throw_error(&err_b)
            }
        }
    }
}

impl<A, E> BaseAction<A> for Result<A, E> {
    fn pure(value: A) -> Self {
        Ok(value)
    }
}

impl<A, E> Error<A, E> for Result<A, E>
where
    Self: Clone,
{
    fn unwrap_error(&self) -> Result<A, E> {
        self.clone()
    }
}

impl<A, E> BaseError<A, E> for Result<A, E> 
where
    E: Clone,
{
    fn throw_error(err: &E) -> Self {
        Err(err.clone())
    }
}

impl<A, E> Mappable<A> for Result<A, E> {}
impl<A, E> Sequential<A> for Result<A, E> {}
impl<A, E> Action<A> for Result<A, E> {}

impl<'a, A, B, ActA, E, F> Error<B, E> for Map<'a, A, ActA, F>
where
    ActA: Error<A, E>,
    F: Fn(&A) -> B,
{
    fn unwrap_error(&self) -> Result<B, E> {
        self.act_a.unwrap_error().map(|value| (self.func)(&value))
    }
}
impl<'b, A, B, ActA, ActB, E> Error<A, E> for First<'b, B, ActA, ActB>
where
    ActA: Error<A, E>,
    ActB: Error<B, E>,
{
    fn unwrap_error(&self) -> Result<A, E> {
        self.act_a.unwrap_error().and_then(|result| {
            self.act_b.unwrap_error()?;
            Ok(result)
        })
    }
}
impl<'a, A, B, ActA, ActB, E> Error<B, E> for Second<'a, A, ActA, ActB>
where
    ActA: Error<A, E>,
    ActB: Error<B, E>,
{
    fn unwrap_error(&self) -> Result<B, E> {
        self.act_a.unwrap_error()?;
        self.act_b.unwrap_error()
    }
}
impl<A, B, ActA, ActB, E> Error<(A, B), E> for Sequence<ActA, ActB>
where
    ActA: Error<A, E>,
    ActB: Error<B, E>,
{
    fn unwrap_error(&self) -> Result<(A, B), E> {
        self.0
            .unwrap_error()
            .and_then(|result_a| self.1.unwrap_error().map(|result_b| (result_a, result_b)))
    }
}
impl<'a, 'b, A, B, C, ActA, ActB, E, F> Error<C, E> for Combine<'a, 'b, A, B, ActA, ActB, F>
where
    ActA: Error<A, E>,
    ActB: Error<B, E>,
    F: Fn(&A, &B) -> C,
{
    fn unwrap_error(&self) -> Result<C, E> {
        self.act_a.unwrap_error().and_then(|result_a| {
            self.act_b
                .unwrap_error()
                .map(|result_b| (self.func)(&result_a, &result_b))
        })
    }
}

impl<'a, 'ab, A, B, ActA, ActB, E, F> Error<B, E> for AndThen<'a, 'ab, A, ActA, ActB, F>
where
    ActA: Error<A, E>,
    ActB: Error<B, E>,
    F: Fn(&A) -> ActB,
{
    fn unwrap_error(&self) -> Result<B, E> {
        self.act_a
            .unwrap_error()
            .and_then(|result| (self.func)(&result).unwrap_error())
    }
}


// #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// enum Either<A, B> {
//     Left(A),
//     Right(B),
// }

// struct RunError<'a, A, ActA> {
//     act_a: ActA,
//     _act_a_type: PhantomData<&'a A>,
// }
// trait Error<E, A>: Action<A> {
//     fn run_error<'a>(self) -> RunError<'a, A, Self> {
//         RunError {
//             act_a: self,
//             _act_a_type: PhantomData,
//         }
//     }
// }

// impl<'a, A, ActA, E> Mappable<Either<E, A>> for RunError<'a, A, ActA> where ActA: Mappable<A> {}
// impl<'a, A, ActA, E> Sequential<Either<E, A>> for RunError<'a, A, ActA> where ActA: Sequential<A> {}
// impl<'a, A, ActA, E> Action<Either<E, A>> for RunError<'a, A, ActA> where ActA: Action<A> {}