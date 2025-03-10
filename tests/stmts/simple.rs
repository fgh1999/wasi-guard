#[allow(unused_imports)]
use wasi_guard::policy::{action::Action::Kill, policy};
use wasi_guard::wasi::*;

policy! {
    default = Kill;
    allow args_get where |_a: i32, _b: i32| true;
    allow proc_exit where |code: i32| code >= 0;
    log proc_exit where |code: i32| code >= 4, |i| i & 0b1 == 0b1;
}

#[test]
fn default() {
    assert_eq!(DEFUALT_ACTION, Kill);
}

#[test]
fn args_get_exists() {
    assert!(WASI_GUARD_ARGS_GET.is_some());
}

#[test]
fn proc_exit_exists() {
    assert!(WASI_GUARD_PROC_EXIT.is_some());
    let actions = WASI_GUARD_PROC_EXIT.as_ref().unwrap().check((0,));
    assert_eq!(
        wasi_guard::policy::action::actions_to_execute(&actions).count(),
        0
    );

    // 5 > 4 and 5 & 0b1 == 0b1
    let actions = WASI_GUARD_PROC_EXIT.as_ref().unwrap().check((5,));
    assert_eq!(actions.len(), 2);
    assert_eq!(
        wasi_guard::policy::action::actions_to_execute(&actions).count(),
        1
    );
}

#[test]
fn environ_get_does_not_exists() {
    assert!(!WASI_GUARD_ENVIRON_GET.is_some());
}
