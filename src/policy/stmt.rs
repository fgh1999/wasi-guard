use wasi_guard_macros::all_tuples;

use super::{
    action::Action,
    bound::{AbiArgBound, PredicateParams},
};
use crate::{
    util::Tuple,
    wasi::{AbiArg, WasiAbiDescriptor},
};

/// If `abi` [satisfies `bound`], then `action`.
pub struct Statement<'desc, Params: Tuple + PredicateParams>
where
    [(); Params::LENGTH]:,
{
    abi: &'desc WasiAbiDescriptor<'desc, { Params::LENGTH }>,
    bound: Option<AbiArgBound<Params>>,
    action: Action,
}

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
            /// Returns `true` if there is no bound.
            #[allow(unused)]
            pub fn check_bound(&self, params: ( $($P,)* )) -> bool {
                use $crate::policy::bound::CheckArgBound;
                self.bound.as_ref().map_or(true, |bound| bound.check(params))
            }
        }
    };
}
all_tuples!(impl_check_bound_for_statement[0,10]: P);

// pub trait Stmt<'s> {
//     fn arg_num(&self) -> usize;
//     fn arg_at(&self, index: usize) -> Option<&'s AbiArg>;
//     // fn bound(&self) -> &'s AbiArgBound<Self::ARG_NUM>;
//     // fn check_bound<Params>(&self, params: Params) -> bool where Params: core::fmt::Debug;
//     fn action(&self) -> &Action;
// }

macro_rules! replace_tt_with_dt {
    ($($tt:tt)*) => {
        $crate::wasi::DefaultAbiArgType
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

mod act_statement {
    #[macro_export]
    macro_rules! statement {
        ($abi:path) => {{
            ($abi).trigger($crate::policy::action::Action::default())
        }};
        ($abi:path $(where)? => $($act_type:tt)+) => {{
            ($abi).trigger($($act_type)+)
        }};
        ($abi:path where $bound:expr => $($act_type:tt)+) => {{
            ($abi).trigger($($act_type)+)
            .when($bound)
        }};
    }
    // Macro_expanded macro_rules are not permitted to be used with absolute path,
    // So here are just the elementary implementations.
    #[macro_export]
    macro_rules! _inner_allow {
        ($abi:path $(where)?) => {{
            $crate::statement!($abi => $crate::policy::action::Action::Allow)
        }};
        ($abi:path where $bound:expr) => {{
            $crate::statement!($abi where $bound => $crate::policy::action::Action::Allow)
        }};
    }
    #[macro_export]
    macro_rules! _inner_kill {
        ($abi:path $(where)?) => {{
            $crate::statement!($abi => $crate::policy::action::Action::Kill)
        }};
        ($abi:path where $bound:expr) => {{
            $crate::statement!($abi where $bound => $crate::policy::action::Action::Kill)
        }};
    }
    #[macro_export]
    macro_rules! _inner_log {
        ($abi:path $(where)?) => {{
            $crate::statement!($abi => $crate::policy::action::Action::Log)
        }};
        ($abi:path where $bound:expr) => {{
            $crate::statement!($abi where $bound => $crate::policy::action::Action::Log)
        }};
    }

    #[macro_export]
    macro_rules! _inner_return_errno {
        ($abi:path $(where)? => $errno:expr) => {{
            const ERRNO: $crate::policy::action::WasiErrno = $errno;
            const ACT: $crate::policy::action::Action = $crate::policy::action::Action::ReturnErrno(ERRNO);
            $crate::statement!($abi => ACT)
        }};
        ($abi:path where $bound:expr => $errno:expr) => {{
            const ERRNO: $crate::policy::action::WasiErrno = $errno;
            const ACT: $crate::policy::action::Action = $crate::policy::action::Action::ReturnErrno(ERRNO);
            $crate::statement!($abi where $bound => ACT)
        }};
    }
}

#[cfg(test)]
mod test {
    use crate::{
        desc_wasi_abi,
        policy::{bound::AbiArgBound, stmt::Action},
        wasi::WasiAbiDescriptor,
    };
    const WASI: WasiAbiDescriptor<2> = desc_wasi_abi!(clock_time_get(clock_id, precision[8]));

    #[test]
    fn claim_statement() {
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

    #[test]
    fn statement_macro() {
        use crate::statement;
        let statement = statement!(WASI => Action::Kill);
        // always `true` on `check_bound` because these is no bound
        assert!(statement.check_bound((1, 0)));
        assert!(statement.check_bound((0 - 1, 0)));
        let statement = statement!(WASI);
        assert_eq!(statement.action, Action::default());

        let statement = statement!(WASI where => Action::Kill);
        // always `true` on `check_bound` because these is no bound
        assert!(statement.check_bound((1, 0)));
        assert!(statement.check_bound((0 - 1, 0)));

        let statement = statement!(WASI where |a: i32, _: bool| a > 0 => Action::Kill);
        assert_eq!(statement.abi.name, "clock_time_get");
        assert_eq!(statement.abi.args.len(), 2);
        assert!(statement.check_bound((1, true)));
        assert!(!statement.check_bound((0 - 1, false)));

        mod tmp {
            use super::*;
            pub const WASI: WasiAbiDescriptor<2> =
                desc_wasi_abi!(clock_time_get(clock_id, precision[8]));
        }
        let _statement = statement!(tmp::WASI);
    }

    #[test]
    fn inner_statement_macros() {
        use crate::{_inner_allow, _inner_kill, _inner_log, _inner_return_errno};
        let allow_stat = _inner_allow!(WASI);
        assert_eq!(allow_stat.action, Action::Allow);
        let kill_stat = _inner_kill!(WASI);
        assert_eq!(kill_stat.action, Action::Kill);
        let log_stat = _inner_log!(WASI where |x: i32, y: i64| x as i64 + y > 0);
        assert_eq!(log_stat.action, Action::Log);
        let ret_stat = _inner_return_errno!(WASI => 23);
        assert_eq!(ret_stat.action, Action::ReturnErrno(23));
    }
}
