use wasi_descriptor::WasiAbiDescriptor;
use wasi_guard_macros::all_tuples;

use super::{
    action::Action,
    bound::{AbiArgBound, PredicateParams},
};
use crate::util::Tuple;

/// If `abi` [satisfies `bound`], then `action`.
#[derive(Clone)]
pub struct Statement<'desc, Params: Tuple + PredicateParams>
where
    [(); Params::LENGTH]:,
{
    abi: &'desc WasiAbiDescriptor<'desc, { Params::LENGTH }>,
    bound: Option<AbiArgBound<'desc, Params>>,
    pub action: Action,
}

impl<'desc, Params: Tuple + PredicateParams> Statement<'desc, Params>
where
    [(); Params::LENGTH]:,
{
    // TODO: into const fn
    pub fn when<NewParams>(
        self,
        bound: impl Into<AbiArgBound<'desc, NewParams>> + 'desc,
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

    // TODO: into const fn
    pub fn and_when(self, other_bound: impl Into<AbiArgBound<'desc, Params>>) -> Self
    where
        Params: Clone,
    {
        let Self { abi, bound, action } = self;
        let other_bound: AbiArgBound<Params> = other_bound.into();
        let bound = match bound {
            None => other_bound,
            Some(b) => b.and(other_bound),
        };
        Statement {
            abi,
            bound: Some(bound),
            action,
        }
    }

    pub const fn trigger(mut self, action: Action) -> Self {
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
            /// Checks if the bound is satisfied by the given parameters.
            /// Returns `true` if there is no bound.
            #[allow(unused)]
            pub fn check_bound(&self, params: ( $($P,)* )) -> bool {
                self.bound.as_ref().map_or(true, |bound| bound.check(params))
            }
        }
    };
}
all_tuples!(impl_check_bound_for_statement[0,10]: P);

macro_rules! replace_tt_with_dt {
    ($($tt:tt)*) => {
        wasi_descriptor::DefaultAbiArgType
    };
}

pub trait Trigger<'initiator> {
    type DefaultOutput;
    fn trigger(&'initiator self, action: Action) -> Self::DefaultOutput;
}
// Use i32 as the default type of predicate parameters
macro_rules! impl_trigger_for_wasi_abi {
    ($($P:ident),*) => {
        impl<'desc> Trigger<'desc> for WasiAbiDescriptor<'desc, {$crate::__count_idents!($($P),*)}> {
            type DefaultOutput = Statement<'desc, ( $(replace_tt_with_dt!($P),)* )>;
            fn trigger(&'desc self, action: Action) -> Self::DefaultOutput {
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
    /// Use [`Action::Kill`][default] as the default action.
    ///
    /// [default]: crate::policy::action::Action::default
    #[macro_export]
    macro_rules! statement {
        ($abi:path) => {{
            use $crate::policy::stmt::Trigger;
            ($abi).trigger($crate::policy::action::Action::default())
        }};
        ($abi:path $(where)? => $($act_type:tt)+) => {{
            use $crate::policy::stmt::Trigger;
            ($abi).trigger($($act_type)+)
        }};
        ($abi:path where $bound:expr => $($act_type:tt)+) => {{
            use $crate::policy::stmt::Trigger;
            ($abi).trigger($($act_type)+)
            .when($bound)
        }};
        ($abi:path where $bound:expr, $($other_bound:expr),+ => $($act_type:tt)+) => {{
            use $crate::policy::stmt::Trigger;
            ($abi).trigger($($act_type)+)
            .when($bound)
            $(.and_when($other_bound))+
        }};
    }

    // Macro-expanded `macro_rules` are not permitted to be used with absolute path,
    // so the followings are just the elementary implementations.

    /// Constructs a [`super::Statement`] with the given ABI descripter([`WasiAbiDescriptor`][desc]) and [`Action::Allow`][action].
    /// Uses `where`-clauses to specify the bounds of the statement.
    ///
    /// [desc]: crate::wasi::WasiAbiDescriptor
    /// [action]: crate::policy::action::Action::Allow
    #[macro_export]
    macro_rules! _inner_allow {
        ($abi:path $(where)?) => {{
            $crate::statement!($abi => $crate::policy::action::Action::Allow)
        }};
        ($abi:path where $($bound:expr),+ $(,)*) => {{
            $crate::statement!($abi where $($bound),* => $crate::policy::action::Action::Allow)
        }};
    }

    /// Constructs a [`super::Statement`] with the given ABI descripter([`WasiAbiDescriptor`][desc]) and [`Action::Kill`][action].
    /// Uses `where`-clauses to specify the bounds of the statement.
    ///
    /// [desc]: crate::wasi::WasiAbiDescriptor
    /// [action]: crate::policy::action::Action::Kill
    #[macro_export]
    macro_rules! _inner_kill {
        ($abi:path $(where)?) => {{
            $crate::statement!($abi => $crate::policy::action::Action::Kill)
        }};
        ($abi:path where $($bound:expr),+ $(,)*) => {{
            $crate::statement!($abi where $($bound),* => $crate::policy::action::Action::Kill)
        }};
    }

    /// Constructs a [`super::Statement`] with the given ABI descripter([`WasiAbiDescriptor`][desc]) and [`Action::Log`][action].
    /// Uses `where`-clauses to specify the bounds of the statement.
    ///
    /// [desc]: crate::wasi::WasiAbiDescriptor
    /// [action]: crate::policy::action::Action::Log
    #[macro_export]
    macro_rules! _inner_log {
        ($abi:path $(where)?) => {{
            $crate::statement!($abi => $crate::policy::action::Action::Log)
        }};
        ($abi:path where $($bound:expr),+ $(,)*) => {{
            $crate::statement!($abi where $($bound),* => $crate::policy::action::Action::Log)
        }};
    }

    /// Constructs a [`super::Statement`] with the given ABI descripter([`WasiAbiDescriptor`][desc]) and [`Action::ReturnErrno`][action].
    /// Uses `where`-clauses to specify the bounds of the statement.
    /// Uses `=>` to specify the errno to return.
    ///
    /// [desc]: crate::wasi::WasiAbiDescriptor
    /// [action]: crate::policy::action::Action::ReturnErrno
    #[macro_export]
    macro_rules! _inner_return_errno {
        ($abi:path $(where)? => $errno:expr) => {{
            const ERRNO: $crate::policy::action::WasiErrno = $errno;
            const ACT: $crate::policy::action::Action = $crate::policy::action::Action::ReturnErrno(ERRNO);
            $crate::statement!($abi => ACT)
        }};
        ($abi:path where $($bound:expr),+ $(,)* => $errno:expr) => {{
            const ERRNO: $crate::policy::action::WasiErrno = $errno;
            const ACT: $crate::policy::action::Action = $crate::policy::action::Action::ReturnErrno(ERRNO);
            $crate::statement!($abi where $($bound),* => ACT)
        }};
    }
}

#[cfg(test)]
mod test {
    use wasi_descriptor::{desc_wasi_abi, WasiAbiDescriptor};

    use crate::policy::{
        bound::AbiArgBound,
        stmt::{Action, Trigger},
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

        let other_bound = |_: i32, b: bool| b;
        let statement = statement!(WASI where |a: i32, _: bool| a > 0, other_bound => Action::Kill);
        assert_eq!(statement.abi.name, "clock_time_get");
        assert_eq!(statement.abi.args.len(), 2);
        assert!(statement.check_bound((1, true)));
        assert!(!statement.check_bound((1, false)));
        assert!(!statement.check_bound((0 - 1, true)));

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
        // With a comma at the end
        let allow_stat = _inner_allow!(
            WASI where |x: i32, y: i64| x as i64 + y > 0, |x: i32, y: i64| x as i64 + y < 256,);
        assert_eq!(allow_stat.action, Action::Allow);

        // Without a comma at the end
        let kill_stat = _inner_kill!(
            WASI where |x: i32, y: i64| x as i64 + y > 0, |x: i32, y: i64| x as i64 + y < 256);
        assert_eq!(kill_stat.action, Action::Kill);

        let log_stat = _inner_log!(
            WASI where |x: i32, y: i64| x as i64 + y > 0);
        assert_eq!(log_stat.action, Action::Log);

        let ret_stat_0 = _inner_return_errno!(WASI => 23);
        assert_eq!(ret_stat_0.action, Action::ReturnErrno(23));
        // With redundant commas
        let _ret_stat_1 = _inner_return_errno!(
            WASI where
                |x: i32, y: i64| x as i64 + y > 0,
                |x: i32, y: i64| x as i64 + y < 256,,
            => 23
        );
    }
}
