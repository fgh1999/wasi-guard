use wasi_guard::policy::policy;
#[allow(unused_imports)]
use wasi_guard::wasi::*;

policy! {
    default = Kill;
    log proc_exit;
}

#[test]
fn proc_exit_exists() {
    assert!(WASI_GUARD_PROC_EXIT.is_some());
    let actions = WASI_GUARD_PROC_EXIT.as_ref().unwrap().check((0,));
    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], wasi_guard::policy::action::Action::Log);
}
