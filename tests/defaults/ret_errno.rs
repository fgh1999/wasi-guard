use wasi_guard::policy::{
    action::{Action::ReturnErrno, WasiErrno},
    policy,
};

const DEFUALT_ERRNO: WasiErrno = 21;

policy! {
    default = ret_err(DEFUALT_ERRNO);
}

#[test]
fn default() {
    assert_eq!(DEFUALT_ACTION, ReturnErrno(DEFUALT_ERRNO));
}
