use std::marker::PhantomData;

use super::{
    Action, AndThen, AndThenCtx, BaseAction, Combine, First, Map, Mappable, Run, Second, Sequence, Sequential,
    TypeBuilder,
};

pub struct BuildError<'e, S, E>(pub PhantomData<&'e E>, pub S);

trait BaseError<A, S, E>: BaseAction<A> + Run<S, A>
where
    S: TypeBuilder<A, Output = Self>,
    Self: Sized,
{
    fn throw_error(err: &E) -> Self;
    fn catch_error<ActA, ActB, F>(action: &ActA, func: F) -> Self
    where
        ActA: Run<S, A>,
        ActB: Run<S, A>,
        F: Fn(&E) -> ActB;
}

impl<'e, A, S, E> TypeBuilder<A> for BuildError<'e, S, E>
where
    S: TypeBuilder<Result<A, E>>,
    S::Output: BaseAction<A>,
{
    type Output = S::Output;
    fn build(value: A) -> Self::Output {
        S::build(Ok(value))
    }
}


impl<A, BaseA, E> Mappable<A> for Result<BaseA, E> {}
impl<A, BaseA, E> Sequential<A> for Result<BaseA, E> where BaseA: BaseAction<A> {}
impl<A, BaseA, E> Action<A> for Result<BaseA, E> where BaseA: BaseAction<A> {}

impl<A, BaseA, E> BaseAction<A> for Result<BaseA, E>
where
    BaseA: BaseAction<A>,
{
    fn pure(value: A) -> Self {
        Ok(BaseA::pure(value))
    }
}

impl<'e, A, S, R, E> Run<BuildError<'e, S, E>, A> for R
where
    R: BaseAction<A> + BaseAction<Result<A, E>>,
    S: TypeBuilder<Result<A, E>, Output = R>,
    Self: Clone,
{
    fn run(&self) -> <BuildError<'e, S, E> as TypeBuilder<A>>::Output {
        self.clone()
    }
}

impl<'e, A, S, R, E> BaseError<A, BuildError<'e, S, E>, E> for R
where
    R: BaseAction<A> + BaseAction<Result<A, E>> + Clone,
    S: TypeBuilder<Result<A, E>, Output = R>,
    E: 'e + Clone,
{
    fn throw_error(err: &E) -> Self {
        S::build(Err(err.clone()))
    }
    fn catch_error<ActA, ActB, F>(action: &ActA, func: F) -> Self
    where
        ActA: Run<BuildError<'e, S, E>, A>,
        ActB: Run<BuildError<'e, S, E>, A>,
        F: Fn(&E) -> ActB,
    {
        unimplemented!()
    }
}

// impl<'a, 'e, A, B, ActA, S, E, F> Run<BuildError<'e, S, E>, B> for Map<'a, A, ActA, F>
// where
//     E: 'e,
//     S: TypeBuilder<Result<A, E>> + TypeBuilder<Result<B, E>>,
//     <S as TypeBuilder<Result<A, E>>>::Output: BaseAction<A>,
//     <S as TypeBuilder<Result<B, E>>>::Output: BaseAction<B>,
//     ActA: Run<BuildError<'e, S, E>, A>,
//     F: Fn(&A) -> B,
//     Map<'a, A, <S as TypeBuilder<Result<A, E>>>::Output, F>: Run<S, Result<B, E>>,
// {
//     fn run(&self) -> <BuildError<'e, S, E> as TypeBuilder<B>>::Output {
//         unimplemented!()
//     }
// }
// impl<'b, 'e, A, B, ActA, ActB, S, E> Run<BuildError<'e, S, E>, A> for First<'b, B, ActA, ActB>
// where
//     E: 'e,
//     S: TypeBuilder<Result<A, E>> + TypeBuilder<Result<B, E>>,
//     <S as TypeBuilder<Result<A, E>>>::Output: BaseAction<A>,
//     <S as TypeBuilder<Result<B, E>>>::Output: BaseAction<B>,
//     ActA: Run<BuildError<'e, S, E>, A>,
//     ActB: Run<BuildError<'e, S, E>, B>,
// {
//     fn run(&self) -> <BuildError<'e, S, E> as TypeBuilder<A>>::Output {
//         unimplemented!()
//         // Disambiguation: the and_then used here is the and_then from std::Result.
//         // self.act_a.run().and_then(|result| {
//         //     self.act_b.run()?;
//         //     Ok(result)
//         // })
//     }
// }
// impl<'a, 'e, A, B, ActA, ActB, S, E> Run<BuildError<'e, S, E>, B> for Second<'a, A, ActA, ActB>
// where
//     E: 'e,
//     S: TypeBuilder<Result<A, E>> + TypeBuilder<Result<B, E>>,
//     <S as TypeBuilder<Result<A, E>>>::Output: BaseAction<A>,
//     <S as TypeBuilder<Result<B, E>>>::Output: BaseAction<B>,
//     ActA: Run<BuildError<'e, S, E>, A>,
//     ActB: Run<BuildError<'e, S, E>, B>,
// {
//     fn run(&self) -> <BuildError<'e, S, E> as TypeBuilder<B>>::Output {
//         unimplemented!()
//         // self.act_a.run();
//         // self.act_b.run()
//     }
// }
// impl<'e, A, B, ActA, ActB, S, E> Run<BuildError<'e, S, E>, (A, B)> for Sequence<ActA, ActB>
// where
//     E: 'e,
//     S: TypeBuilder<Result<A, E>> + TypeBuilder<Result<B, E>> + TypeBuilder<Result<(A, B), E>>,
//     <S as TypeBuilder<Result<A, E>>>::Output: BaseAction<A>,
//     <S as TypeBuilder<Result<B, E>>>::Output: BaseAction<B>,
//     <S as TypeBuilder<Result<(A, B), E>>>::Output: BaseAction<(A, B)>,
//     ActA: Run<BuildError<'e, S, E>, A>,
//     ActB: Run<BuildError<'e, S, E>, B>,
//     Sequence<<S as TypeBuilder<Result<A, E>>>::Output, <S as TypeBuilder<Result<B, E>>>::Output>:
//         Run<S, Result<(A, B), E>>,
// {
//     fn run(&self) -> <BuildError<'e, S, E> as TypeBuilder<(A, B)>>::Output {
//         unimplemented!()

//         // match self.0.run() {
//         //     Err(err_a) => Err(err_a),
//         //     Ok(action_inner_a) => match self.1.run() {
//         //         Err(err_b) => Err(err_b),
//         //         Ok(action_inner_b) =>
//         //             Ok(action_inner_a.sequence(action_inner_b).run()),
//         //     }
//         // }
//     }
// }
// impl<'a, 'b, 'e, A, B, C, ActA, ActB, S, E, F> Run<BuildError<'e, S, E>, C>
//     for Combine<'a, 'b, A, B, ActA, ActB, F>
// where
//     E: 'e,
//     S: TypeBuilder<Result<A, E>> + TypeBuilder<Result<B, E>> + TypeBuilder<Result<C, E>>,
//     <S as TypeBuilder<Result<A, E>>>::Output: BaseAction<A>,
//     <S as TypeBuilder<Result<B, E>>>::Output: BaseAction<B>,
//     <S as TypeBuilder<Result<C, E>>>::Output: BaseAction<C>,
//     ActA: Run<BuildError<'e, S, E>, A>,
//     ActB: Run<BuildError<'e, S, E>, B>,
//     F: Fn(&A, &B) -> C,
//     Combine<
//         'a,
//         'b,
//         A,
//         B,
//         <S as TypeBuilder<Result<A, E>>>::Output,
//         <S as TypeBuilder<Result<B, E>>>::Output,
//         F,
//     >: Run<S, Result<C, E>>,
// {
//     fn run(&self) -> <BuildError<'e, S, E> as TypeBuilder<C>>::Output {
//         unimplemented!()
//         // match self.act_a.run() {
//         //     Err(err_a) => Err(err_a),
//         //     Ok(action_inner_a) => match self.act_b.run() {
//         //         Err(err_b) => Err(err_b),
//         //         Ok(action_inner_b) =>
//         //             Ok(action_inner_a.combine(action_inner_b, self.func).run()),
//         //     }
//         // }
//     }
// }

// impl<'a, 'ab, 'e, A, B, ActA, ActB, S, E, F> Run<BuildError<'e, S, E>, B>
//     for AndThen<'a, 'ab, A, ActA, ActB, F>
// where
//     E: Clone,
//     S: TypeBuilder<Result<A, E>> + TypeBuilder<Result<B, E>>,
//     <S as TypeBuilder<Result<B, E>>>::Output: 'ab + BaseAction<B>,
//     ActA: Run<S, Result<A, E>> + Action<A>,
//     ActB: Run<S, Result<B, E>> + Action<B>,
//     F: Fn(&A) -> ActB,
//     AndThen<'a, 'ab, A, <S as TypeBuilder<Result<A, E>>>::Output, <S as TypeBuilder<Result<B, E>>>::Output, F>: Run<BuildError<'e, S, E>, B>,
// {
//     fn run(&self) -> <BuildError<'e, S, E> as TypeBuilder<B>>::Output {
//         self.act_a.run().and_then_ctx(self.func, |func, result| {
//             match result {
//                 Err(err) => <S as TypeBuilder<Result<B, E>>>::build(Err(err.clone())),
//                 Ok(ok) => unimplemented!(),
//             }
//         })
//         .run()
//     }
// }

impl<'ab, 'e, A, B, ActA, ActB, S, E, Ctx> Run<BuildError<'e, S, E>, B>
    for AndThenCtx<'ab, A, ActA, ActB, Ctx>
where
    E: 'e + Clone,
    ActA: Run<S, Result<A, E>> + Action<A>,
    ActB: Run<S, Result<B, E>> + BaseAction<Result<B, E>> + BaseAction<B>,
    S: TypeBuilder<Result<A, E>> + TypeBuilder<Result<B, E>, Output = ActB>,
    <S as TypeBuilder<Result<B, E>>>::Output: 'ab + BaseAction<B>,
    AndThenCtx<'ab, Result<A, E>, ActA, <S as TypeBuilder<Result<B, E>>>::Output, (Ctx, fn(&Ctx, &A) -> ActB)>: Run<BuildError<'e, S, E>, B>,
{
    fn run(&self) -> <BuildError<'e, S, E> as TypeBuilder<B>>::Output {
        self.act_a.and_then_ctx((self.context, self.func), |(context, func), result_a| {
            match result_a {
                Err(err_a) => <S as TypeBuilder<Result<B, E>>>::build(Err(err_a.clone())),
                Ok(result_b) => func(context, result_b),
            }
        })
        .run()
    }
}




