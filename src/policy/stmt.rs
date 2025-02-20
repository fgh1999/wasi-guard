use wasi_guard_macros::all_tuples;

use super::{
    action::Action,
    bound::{AbiArgBound, PredicateParams},
};
use crate::{
    util::Tuple,
    wasi::{AbiArg, WasiAbiDescriptor},
};

// If `abi` satisfies `bound`, then `action`.
// statement 之间只能合并（AND）
pub struct Statement<'desc, Params: Tuple + PredicateParams>
where
    [(); Params::LENGTH]:,
{
    abi: &'desc WasiAbiDescriptor<'desc, { Params::LENGTH }>,
    bound: Option<AbiArgBound<Params>>,
    action: Action,
}
// claim a statement:
// action { $(abi $(where bound)?),+ }

// macro_rules! statement {
//     {$action:ident { $($abi:ident $(where $bound:expr)?),+ } } => {

//     };
// }
impl<'desc, Params: Tuple + PredicateParams> Statement<'desc, Params>
where
    [(); Params::LENGTH]:,
{
    pub fn when<NewParams>(
        self,
        bound: impl Into<AbiArgBound<NewParams>> + 'desc,
    ) -> Statement<'desc, NewParams>
    where
        NewParams: Tuple + PredicateParams,
        [(); NewParams::LENGTH - Params::LENGTH]:,
        [(); Params::LENGTH - NewParams::LENGTH]:,
    {
        let Self { abi, action, .. } = self;
        Statement {
            // SAFETY: NewParams::LENGTH == Params::LENGTH is guaranteed by the where clause
            abi: unsafe {
                core::mem::transmute::<
                    &WasiAbiDescriptor<'_, { Params::LENGTH }>,
                    &WasiAbiDescriptor<'desc, { NewParams::LENGTH }>,
                >(abi)
            },
            bound: Some(bound.into()),
            action,
        }
    }

    pub fn trigger(mut self, action: Action) -> Statement<'desc, Params> {
        self.action = action;
        self
    }
}

macro_rules! impl_check_bound_for_statement {
    ($($P:ident),*) => {
        impl<'desc, $($P,)*> Statement<'desc, ( $($P,)* )>
        where [(); <( $($P,)* )>::LENGTH]:,
            ( $($P,)* ) : $crate::policy::bound::PredicateParams,
            ( $($P,)* ) : $crate::util::Tuple,
        {
            #[allow(unused)]
            fn check_bound(&self, params: ( $($P,)* )) -> bool {
                use $crate::policy::bound::CheckArgBound;
                self.bound.as_ref().map_or(true, |bound| bound.check(params))
            }
        }
    };
}
all_tuples!(impl_check_bound_for_statement[0,10]: P);

pub trait Stmt<'s> {
    fn arg_num(&self) -> usize;
    fn arg_at(&self, index: usize) -> Option<&'s AbiArg>;
    // fn bound(&self) -> &'s AbiArgBound<Self::ARG_NUM>;
    // fn check_bound<Params>(&self, params: Params) -> bool where Params: core::fmt::Debug;
    fn action(&self) -> &Action;
}

// pub trait ClaimStatement<Params: Tuple + Debug> {
//     fn trigger(&self, action: Action) -> Statement<'_, Params>
//     where
//         [(); Params::LENGTH]:;
// }
// #[allow(unused)]
// macro_rules! impl_claim_statement_for_tuple {
//     ($($P:ident),*) => {
//         impl<$($P,)*> ClaimStatement<( $($P,)* )> for WasiAbiDescriptor<'_, {<( $($P,)* )>::LENGTH}>
//         where ( $($P,)* ) : $crate::policy::bound::PredicateParams,
//             ( $($P,)* ) : $crate::util::Tuple,
//         {
//             fn trigger(
//                 &self,
//                 action: Action,
//             ) -> Statement<'_, ( $($P,)* )> where [(); <( $($P,)* )>::LENGTH]: {
//                 Statement {
//                     abi: self,
//                     bound: None,
//                     action,
//                 }
//             }
//         }
//     };
// }
// all_tuples!(impl_claim_statement_for_tuple[0,10]: P);
macro_rules! replace_tt_with_dt {
    ($($tt:tt)*) => {
        i32
    };
}
// Use i32 as the default type of predicate parameters
macro_rules! impl_trigger_for_wasi_abi {
    ($($P:ident),*) => {
        impl WasiAbiDescriptor<'_,{$crate::__count_idents!($($P),*)}> {
            pub fn trigger(&self, action: Action) -> Statement<'_, ( $(replace_tt_with_dt!($P),)* )> {
                Statement {
                    abi: self,
                    bound: None,
                    action,
                }
            }
        }
    };
}
all_tuples!(impl_trigger_for_wasi_abi[0,10]: P);

#[cfg(test)]
mod test {
    use crate::policy::{bound::AbiArgBound, stmt::Action};

    #[test]
    fn claim_statement() {
        use crate::wasi::WasiAbiDescriptor;

        const WASI: WasiAbiDescriptor<2> = desc_wasi_abi!(clock_time_get(clock_id, precision[8]));
        let statement = WASI.trigger(Action::Allow);
        assert_eq!(statement.abi.name, "clock_time_get");

        let bound = |a: i32, b: u64| -> bool { a > 0 && b <= 1 << 8 };
        let bound: AbiArgBound<(i32, u64)> = bound.into();
        let statement = statement.when(bound);
        assert_eq!(statement.abi.args.len(), 2);
        assert_eq!(statement.abi.name, "clock_time_get");
        assert!(statement.check_bound((1, 233)));
        assert!(!statement.check_bound((0, 1 << 9)));

        // Reinterpret the statement with a newly-typed bound
        let bound = |a: i32, new_b: u32| -> bool { a > 0 && new_b <= 1 << 8 };
        let bound: AbiArgBound<(i32, u32)> = bound.into();
        let statement = statement.when(bound);
        assert_eq!(statement.abi.name, "clock_time_get");
        assert!(statement.check_bound((1, 233)));
        assert!(!statement.check_bound((0, 1 << 9)));

        // chain-style
        let statement = WASI
            .trigger(Action::Allow)
            .when(|a: i32, _: bool| a > 0)
            .trigger(Action::Kill);
        assert_eq!(statement.abi.name, "clock_time_get");
        assert_eq!(statement.abi.args.len(), 2);
        assert!(statement.check_bound((1, true)));
        assert!(!statement.check_bound((0 - 1, false)));
    }
}
