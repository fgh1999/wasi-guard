use wasi_descriptor::desc_wasi_abi;
use wasi_guard::policy::{action::Action, bound::AbiArgBound, Trigger};

#[test]
fn full_statement() {
    let wasi = desc_wasi_abi!(clock_time_get(clock_id, precision[8]));
    let statement = wasi.trigger(Action::Allow);

    let bound = |a: i32, b: u64| -> bool { a > 0 && b <= 1 << 8 };
    let statement = statement.when(AbiArgBound::from(bound));
    assert!(statement.check_bound((1, 233)));
    assert!(!statement.check_bound((0, 1 << 9)));
}
