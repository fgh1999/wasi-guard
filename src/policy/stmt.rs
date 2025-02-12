use wasi_guard_macros::all_tuples;

use super::{action::Action, bound::CheckArgBound};
use crate::wasi::{AbiArg, WasiAbiDescriptor};

// If `abi` satisfies `bound`, then `action`.
// statement 之间只能合并（AND）
pub struct Statement<'s, 'desc, const ARG_NUM: usize>
where
    'desc: 's,
{
    abi: &'s WasiAbiDescriptor<'desc, ARG_NUM>,
    // bound: Box<dyn CheckArgBound>,
    action: Action,
}
// claim a statement:
// action { $(abi $(where bound)?),+ }

macro_rules! statement {
    {$action:ident { $($abi:ident $(where $bound:expr)?),+ } } => {

    };
}

pub trait Stmt<'s> {
    fn arg_num(&self) -> usize;
    fn arg_at(&self, index: usize) -> Option<&'s AbiArg>;
    // fn bound(&self) -> &'s AbiArgBound<Self::ARG_NUM>;
    // fn check_bound<Params>(&self, params: Params) -> bool where Params: core::fmt::Debug;
    fn action(&self) -> &Action;
}

trait TupleLength {
    const TYPE_LENGTH: usize;
}
macro_rules! impl_tuple_length {
    ($($t:ident),*) => {
        impl<$($t),*> TupleLength for ($($t,)*) {
            const TYPE_LENGTH: usize = $crate::__count_idents!($($t),*);
        }
    };
}
all_tuples!(impl_tuple_length[0, 10]: T);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tuple_length() {
        assert_eq!(<() as TupleLength>::TYPE_LENGTH, 0);
        assert_eq!(<(i32,) as TupleLength>::TYPE_LENGTH, 1);
        struct Struct;
        assert_eq!(<(i32, i64, Struct) as TupleLength>::TYPE_LENGTH, 3);
    }
}
