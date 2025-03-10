use wasi_guard::policy::policy;
#[allow(unused_imports)]
use wasi_guard::wasi::*;
extern crate wasip1;
use wasi_guard::policy::action::Action;
use wasip1::{ERRNO_ACCES, ERRNO_PERM};

policy! {
    default = allow;

    ret_errno(ERRNO_PERM.raw()) sock_recv_from;
    ret_errno(ERRNO_PERM.raw()) sock_send_to;

    // disable most FS operations
    ret_errno(ERRNO_ACCES.raw()) fd_write where
        |fd: u32, _: u32, _: u32, _: u32| fd > 2;
    // ret_errno(ERRNO_ACCES.raw()) poll_oneoff;
    ret_errno(ERRNO_PERM.raw()) fd_advise;
    ret_errno(ERRNO_PERM.raw()) fd_allocate;
    // ret_errno(ERRNO_ACCES.raw()) fd_close;
    ret_errno(ERRNO_PERM.raw()) fd_datasync;
    // ret_errno(ERRNO_ACCES.raw()) fd_fdstat_get;
    // ret_errno(ERRNO_ACCES.raw()) fd_fdstat_set_flags;
    ret_errno(ERRNO_PERM.raw()) fd_fdstat_set_rights;
    ret_errno(ERRNO_PERM.raw()) fd_filestat_get;
    ret_errno(ERRNO_PERM.raw()) fd_filestat_set_size;
    ret_errno(ERRNO_PERM.raw()) fd_filestat_set_times;
    ret_errno(ERRNO_PERM.raw()) fd_pread;
    ret_errno(ERRNO_PERM.raw()) fd_prestat_get;
    ret_errno(ERRNO_PERM.raw()) fd_prestat_dir_name;
    ret_errno(ERRNO_PERM.raw()) fd_pwrite;
    ret_errno(ERRNO_PERM.raw()) fd_read;
    ret_errno(ERRNO_PERM.raw()) fd_readdir;
    ret_errno(ERRNO_PERM.raw()) fd_renumber;
    ret_errno(ERRNO_PERM.raw()) fd_seek;
    ret_errno(ERRNO_PERM.raw()) fd_sync;
    ret_errno(ERRNO_PERM.raw()) fd_tell;

    ret_errno(ERRNO_PERM.raw()) path_create_directory;
    ret_errno(ERRNO_PERM.raw()) path_filestat_get;
    ret_errno(ERRNO_PERM.raw()) path_filestat_set_times;
    ret_errno(ERRNO_PERM.raw()) path_link;
    ret_errno(ERRNO_PERM.raw()) path_open;
    ret_errno(ERRNO_PERM.raw()) path_readlink;
    ret_errno(ERRNO_PERM.raw()) path_remove_directory;
    ret_errno(ERRNO_PERM.raw()) path_rename;
    ret_errno(ERRNO_PERM.raw()) path_symlink;
    ret_errno(ERRNO_PERM.raw()) path_unlink_file;
}

fn main() {
    // specified guard
    let params: fd_prestat_dir_name_params_default_t = (0, 0, 0);
    let actions = WASI_GUARD_FD_PRESTAT_DIR_NAME
        .as_ref()
        .map(|guard| guard.check(params));
    assert!(actions.is_some());
    assert_eq!(actions.unwrap()[0], Action::ReturnErrno(ERRNO_PERM.raw()));

    // default guard
    let params: proc_exit_params_default_t = (0,);
    assert!(WASI_GUARD_PROC_EXIT
        .as_ref()
        .map(|guard| guard.check(params))
        .is_none());
}
