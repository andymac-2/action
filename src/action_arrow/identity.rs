use super::{Action, AndThen, First, FromFunc, Id, Pure, Second, Sequence, Tee};

pub struct LiftIdentity<A>(pub A);

pub trait Identity: Action {
    fn run(&self, arg: &Self::Input) -> Self::Output;
}

impl<'a, 'b, A, B, F> Identity for FromFunc<'a, 'b, A, B, F>
where
    F: Fn(&A) -> B,
{
    fn run(&self, arg: &Self::Input) -> Self::Output {
        (self.func)(arg)
    }
}

impl<'a, A> Identity for Id<'a, A>
where
    A: Clone,
{
    fn run(&self, arg: &Self::Input) -> Self::Output {
        arg.clone()
    }
}

impl<'a, A> Identity for Pure<'a, A>
where
    A: Clone,
{
    fn run(&self, _arg: &Self::Input) -> Self::Output {
        self.0.clone()
    }
}


impl<'a, 'b, 'i, ActA, ActB> Identity for AndThen<'a, 'b, ActA, ActB>
where
    ActA: Identity,
    ActB: Identity<Input = ActA::Output>,
{
    fn run(&self, arg: &Self::Input) -> Self::Output {
        let middle = self.act_a.run(arg);
        self.act_b.run(&middle)
    }
}

impl<'a, 'b, 'ia, 'ib, ActA, ActB> Identity for Sequence<'a, 'b, 'ia, 'ib, ActA, ActB>
where
    ActA: Identity,
    ActB: Identity,
{
    fn run(&self, (arg_a, arg_b): &Self::Input) -> Self::Output {
        let a = self.act_a.run(arg_a);
        let b = self.act_b.run(arg_b);
        (a, b)
    }
}

impl<'a, 'ia, 'ib, ActA, B> Identity for First<'a, 'ia, 'ib, ActA, B>
where
    ActA: Identity,
    B: Clone,
{
    fn run(&self, args: &Self::Input) -> Self::Output {
        let a = self.act_a.run(args.0);
        let b = args.1.clone();
        (a, b)
    }
}

impl<'b, 'ia, 'ib, A, ActB> Identity for Second<'b, 'ia, 'ib, A, ActB>
where
    ActB: Identity,
    A: Clone,
{
    fn run(&self, args: &Self::Input) -> Self::Output {
        let a = args.0.clone();
        let b = self.act_b.run(args.1);
        (a, b)
    }
}

impl<'a, 'b, ActA, ActB> Identity for Tee<'a, 'b, ActA, ActB>
where
    ActA: Identity,
    ActB: Identity<Input = ActA::Input>,
{
    fn run(&self, arg: &Self::Input) -> Self::Output {
        let a = self.act_a.run(arg);
        let b = self.act_b.run(arg);
        (a, b)
    }
}