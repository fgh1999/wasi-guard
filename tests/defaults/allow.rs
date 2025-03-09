use wasi_guard::policy::{action::Action::Allow, policy};

policy! {
    default = allow;
}

#[test]
fn default() {
    assert_eq!(DEFUALT_ACTION, Allow);
}
