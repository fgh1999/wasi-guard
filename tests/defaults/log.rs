use wasi_guard::policy::{action::Action::Log, policy};

policy! {
    default = log;
}

#[test]
fn default() {
    assert_eq!(DEFUALT_ACTION, Log);
}
