pub mod action;
pub mod bound;
pub mod stmt;

use alloc::vec::Vec;

pub use action::Action;
use bound::PredicateParams;
pub use lazy_static::lazy_static;
use smallvec::SmallVec;
use stmt::Statement;
pub use stmt::Trigger;
use wasi_guard_macros::all_tuples;
pub use wasi_guard_macros::policy;

use crate::util::Tuple;

/// The recommended number of statements that can be carried for one [`WasiGuard`].
/// Further statements will be ignored.
pub const STMT_EACH_GUARD: usize = 2;

/// One guard for each WASI ABI.
/// Represents a security policy for a specific WASI ABI,
/// containing multiple statements that share the same parameter types.
/// Each statement consists of a predicate and an action to be taken
/// when the predicate is satisfied.
pub struct WasiGuard<'desc, Params: Tuple + PredicateParams + Clone> {
    statements: SmallVec<[Statement<'desc, Params>; STMT_EACH_GUARD]>,
}

impl<'desc, Params: Tuple + PredicateParams + Clone> From<Vec<Statement<'desc, Params>>>
    for WasiGuard<'desc, Params>
{
    fn from(statements: Vec<Statement<'desc, Params>>) -> Self {
        Self {
            statements: statements.into(),
        }
    }
}
impl<'desc, Params: Tuple + PredicateParams + Clone> WasiGuard<'desc, Params> {
    pub fn from_arr<const N: usize>(statements: [Statement<'desc, Params>; N]) -> Self {
        Self {
            statements: statements.to_vec().into(),
        }
    }
}
macro_rules! impl_from_arr_to_wasi_guard {
    ($($N:literal),*) => {
        $(
            impl<'desc, Params: Tuple + PredicateParams + Clone> From<[Statement<'desc, Params>; $N]>
                for WasiGuard<'desc, Params>
            {
                fn from(statements: [Statement<'desc, Params>; $N]) -> Self {
                    Self::from_arr(statements)
                }
            }
        )*
    };
}
impl_from_arr_to_wasi_guard!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);

/// The recommended number of actions that can be taken for one [`WasiGuard`].
pub const ACTION_NUM: usize = STMT_EACH_GUARD;

macro_rules! impl_check_for_wasi_guard {
    ($($P:ident),*) => {
        impl<'desc, $($P,)*> WasiGuard<'desc, ( $($P,)* )>
        where
            ( $($P,)* ) : $crate::policy::bound::PredicateParams,
            ( $($P,)* ) : $crate::util::Tuple + 'desc,
            ( $($P,)* ) : ::core::clone::Clone,
        {
            /// Checks if the bounds is satisfied by the given parameters,
            /// and returns the actions that should be taken.
            #[allow(unused)]
            pub fn check(&self, params: ( $($P,)* )) -> smallvec::SmallVec<[Action; ACTION_NUM]> {
                self.statements.iter().filter_map(|stmt| {
                    if stmt.check_bound(params.clone()) {
                        Some(stmt.action)
                    } else {
                        None
                    }
                }).collect()
            }
        }
    };
}
all_tuples!(impl_check_for_wasi_guard[0,10]: P);

#[cfg(test)]
mod test {
    use alloc::vec::Vec;

    use wasi_descriptor::{desc_wasi_abi, WasiAbiDescriptor};

    use crate::policy::WasiGuard;

    const WASI: WasiAbiDescriptor<2> = desc_wasi_abi!(clock_time_get(clock_id, precision[8]));

    #[test]
    fn action_filter() {
        use crate::{_inner_allow, _inner_kill};
        let statements = [
            _inner_allow!(WASI where |x: i32, y: i64| x > 0 && y > 0),
            _inner_kill!(WASI where |x: i32, y: i64| x as i64 + y > 256),
        ];
        let guard: WasiGuard<_> = statements.into();

        let actions = guard.check((1, 2));
        assert_eq!(actions.len(), 1);
        use crate::policy::action::Action;
        assert!(actions.iter().all(|action| action == &Action::Allow));

        let actions = guard.check((1, 256));
        assert_eq!(actions.len(), 2);

        let actions: Vec<_> = crate::policy::action::actions_to_execute(&actions).collect();
        assert!(actions.len() == 1);
        assert!(actions.iter().all(|&&action| action != Action::Allow));
    }

    #[test]
    fn overflowing_statements() {
        use crate::{_inner_allow, _inner_kill, _inner_log};
        // The number of statements can be more than `ACTION_NUM`.
        let statements = [
            _inner_allow!(WASI where |x: i32, y: i64| x > 0 && y > 0),
            _inner_kill!(WASI where |x: i32, y: i64| x as i64 + y > 256),
            // -- The following statements will NOT be ignored.
            // -- But the checking performance may be affected.
            _inner_log!(WASI where |x: i32, y: i64| x > 0 && y > 1),
            _inner_kill!(WASI where |x: i32, y: i64| x as i64 + y > 256),
            _inner_allow!(WASI where |a: i32, b: i64| a > 0 && b > 0),
            _inner_kill!(WASI where |a: i32, b: i64| a as i64 + b > 256),
        ];
        assert!(statements.len() > crate::policy::ACTION_NUM);
        let guard: WasiGuard<_> = statements.to_vec().into();

        let actions = guard.check((1, 2));
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0], crate::policy::action::Action::Allow);
        assert_eq!(actions[1], crate::policy::action::Action::Log);
        assert_eq!(actions[2], crate::policy::action::Action::Allow);
    }

    lazy_static::lazy_static! {
        pub static ref LAZY_GUARD: WasiGuard<'static, (i32, i64)> =
        WasiGuard::from_arr([crate::_inner_allow!(WASI where |x: i32, y: i64| x > 0 && y > 0)]);
    }
    #[test]
    fn lazy_static_guard() {
        let actions = LAZY_GUARD.check((1, 2));
        assert!(actions
            .iter()
            .all(|action| action == &crate::policy::action::Action::Allow));
    }
}
