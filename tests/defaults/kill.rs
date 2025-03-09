use wasi_guard::policy::{action::Action::Kill, policy};

policy! {
    default = kill;
}

#[test]
fn default() {
    assert_eq!(DEFUALT_ACTION, Kill);
}
