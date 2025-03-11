use std::sync::LazyLock;

use wasi_descriptor::{desc_wasi_abi, WasiAbiDescriptor};
const WASI: WasiAbiDescriptor<2> = desc_wasi_abi!(clock_time_get(clock_id, precision[8]));

use wasi_guard::{_inner_allow, policy::WasiGuard};
static GUARD: LazyLock<WasiGuard<(i32, i64)>> = LazyLock::new(|| {
    WasiGuard::from_arr([_inner_allow!(WASI where |x: i32, y: i64| x > 0 && y > 0)])
});

#[test]
fn static_guard() {
    let actions = GUARD.check((1, 2));
    assert!(actions
        .iter()
        .all(|action| action == &wasi_guard::policy::action::Action::Allow));
}
