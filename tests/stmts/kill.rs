use wasi_guard::policy::policy;
#[allow(unused_imports)]
use wasi_guard::wasi::*;

policy! {
    default = allow;
    log sched_yield;
    kill sched_yield;
    // kill with bounds => this is NOT a `must_be_killed` WASI
    kill proc_exit where |code: u32| code == 3;
}

#[test]
fn got_killed() {
    assert_eq!(MUST_BE_KILLED_WASIS.len(), 1);
    assert!(MUST_BE_KILLED_WASIS.iter().any(|wasi_name| wasi_name == &"sched_yield"));
    assert!(!MUST_BE_KILLED_WASIS.iter().any(|wasi_name| wasi_name == &"proc_eixt"));
}
