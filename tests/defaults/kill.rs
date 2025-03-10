use wasi_guard::{
    policy::{action::Action::Kill, policy},
    wasi::WASI_NAMES,
};

policy! {
    default = kill;
}

#[test]
fn default() {
    assert_eq!(DEFUALT_ACTION, Kill);
}

#[test]
fn got_killed_anyway() {
    assert_eq!(MUST_BE_KILLED_WASIS.len(), WASI_NAMES.len());
}
