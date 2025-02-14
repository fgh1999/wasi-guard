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

// macro_rules! statement {
//     {$action:ident { $($abi:ident $(where $bound:expr)?),+ } } => {

//     };
// }

pub trait Stmt<'s> {
    fn arg_num(&self) -> usize;
    fn arg_at(&self, index: usize) -> Option<&'s AbiArg>;
    // fn bound(&self) -> &'s AbiArgBound<Self::ARG_NUM>;
    // fn check_bound<Params>(&self, params: Params) -> bool where Params: core::fmt::Debug;
    fn action(&self) -> &Action;
}
