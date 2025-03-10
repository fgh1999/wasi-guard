//! Descriptors for WASI preview 1.

#[allow(unused_imports)]
use wasi_descriptor::{desc_wasi_abi, WasiAbiDescriptor};

use crate::declare_wasi_abis;

/// Address type for WASM linear memory, i.e.,
/// the offset from the base address of the WASM linear memory.
pub type Waddr = u32;

/// Size type for items in WASM linear memory.
pub type Size = u32;

/// File Descriptor.
pub type Fd = u32;

declare_wasi_abis!(
    args_get(arg0, arg1);
    args_sizes_get(arg0, arg1);

    clock_res_get(arg0, arg1);
    clock_time_get(clockid: u32, precision: u64, ret_addr: Waddr);

    environ_get(env_addr: Waddr, env_buf_addr: Waddr);
    environ_sizes_get(env_count_addr: Waddr, env_buf_size_addr: Waddr);

    proc_exit(exitcode: u32);
    proc_raise(sig);
    sched_yield;

    random_get(buf_addr: Waddr, buf_len: Size);

    // FS
    poll_oneoff(
        in_ptr: Waddr,
        out_ptr: Waddr,
        nsubscriptions: Size,
        nevents_ptr: Waddr,
    );
    fd_advise(arg0, arg1: i64, arg2: i64, arg3);
    fd_allocate(arg0, arg1: i64, arg2: i64);
    fd_close(fd: Fd);
    fd_datasync(arg0);
    fd_fdstat_get(fd: Fd, metadata_ptr: Waddr);
    fd_fdstat_set_flags(fd: Fd, flags: u16);
    fd_fdstat_set_rights(arg0, arg1: i64, arg2: i64);
    fd_filestat_get(arg0, arg1);
    fd_filestat_set_size(arg0, arg1: i64);
    fd_filestat_set_times(arg0, arg1, arg2, arg3);
    fd_pread(arg0, arg1, arg2, arg3: i64, arg4);
    fd_prestat_get(fd: Fd, prestat_ptr: Waddr);
    fd_prestat_dir_name(arg0, arg1, arg2);
    fd_pwrite(arg0, arg1, arg2, arg3: i64, arg4);
    fd_read(arg0, arg1, arg2, arg3);
    fd_readdir(arg0, arg1, arg2, arg3: i64, arg4);
    fd_renumber(arg0, arg1);
    fd_seek(arg0, arg1: i64, arg2, arg3);
    fd_sync(fd: Fd);
    fd_tell(arg0, arg1);
    fd_write(fd: Fd, iovs_addr: Waddr, iovs_len: Size, bytes_written_ptr: Waddr);
    path_create_directory(arg0, arg1, arg2);
    path_filestat_get(arg0, arg1, arg2, arg3, arg4);
    path_filestat_set_times(arg0, arg1, arg2, arg3, arg4: i64, arg5: i64, arg6);
    path_link(arg0, arg1, arg2, arg3, arg4, arg5, arg6);
    path_open(arg0, arg1, arg2, arg3, arg4, arg5: i64, arg6: i64, arg7, arg8);
    path_readlink(arg0, arg1, arg2, arg3, arg4, arg5);
    path_remove_directory(arg0, arg1, arg2);
    path_rename(arg0, arg1, arg2, arg3, arg4, arg5);
    path_symlink(arg0, arg1, arg2, arg3, arg4);
    path_unlink_file(arg0, arg1, arg2);
);

// Network
declare_wasi_abis!(
    sock_recv(
        fd: Fd,
        ri_data_ptr: Waddr,
        ri_data_len: Size,
        ri_flags,
        ro_data_len_ptr: Waddr,
        ro_flags_ptr: Waddr,
    );
    sock_send(
        fd: Fd,
        si_data_ptr: Waddr,
        si_data_len: Size,
        si_flags,
        so_data_len_ptr: Waddr,
    );
    sock_shutdown(fd: Fd, how);
);

macro_rules! add_wasi_names {
    ( $($wasi_name:ident),* ) => {{
        [
            "args_get","args_sizes_get",
            "clock_res_get","clock_time_get",
            "environ_get","environ_sizes_get",
            "proc_exit","proc_raise","sched_yield",
            "random_get",

            "poll_oneoff","fd_advise","fd_allocate","fd_close","fd_datasync",
            "fd_fdstat_get","fd_fdstat_set_flags","fd_fdstat_set_rights",
            "fd_filestat_get","fd_filestat_set_size","fd_filestat_set_times",
            "fd_pread","fd_prestat_get","fd_prestat_dir_name","fd_pwrite",
            "fd_read","fd_readdir","fd_renumber","fd_seek","fd_sync","fd_tell",
            "fd_write","path_create_directory","path_filestat_get",
            "path_filestat_set_times","path_link","path_open","path_readlink",
            "path_remove_directory","path_rename","path_symlink","path_unlink_file",

            "sock_recv","sock_send","sock_shutdown","sock_accept",
            $(stringify!($wasi_name)),*
        ]
    }};
}

cfg_if::cfg_if! {
    if #[cfg(feature = "wasmedge-sock")] {
        declare_wasi_abis!(
            sock_listen(fd: Fd, backlog: Size);
            sock_accept(fd: Fd, accepted_fd_ptr: Waddr);
            sock_bind(fd: Fd, addr_buf_ptr: Waddr, port_num: u32);
            sock_connect(fd: Fd, addr_ptr: Waddr, port_num: u32);
            sock_open(addr_family: u8, sock_type: u8, fd_ptr: Waddr);
            sock_recv_from(
                fd: Fd,
                ri_data_ptr: Waddr,
                ri_data_len: Size,
                src_addr_ptr: Waddr,
                ri_flags,
                src_port_ptr: Waddr,
                ro_data_len_ptr: Waddr,
                ro_flags_ptr: Waddr,
            );
            sock_send_to(
                fd: Fd,
                si_data_ptr: Waddr,
                si_data_len: Size,
                dst_addr_ptr: Waddr,
                dst_port: u32,
                si_flags,
                so_data_len_ptr: Waddr,
            );
            sock_getpeeraddr(
                fd: Fd,
                peeraddr_ptr: Waddr,
                peeraddr_type_ptr: Waddr,
                peerport_ptr: Waddr,
            );
            sock_getlocaladdr(
                fd: Fd,
                localaddr_ptr: Waddr,
                localaddr_type_ptr: Waddr,
                localport_ptr: Waddr,
            );
            sock_getsockopt(
                fd: Fd,
                level,
                name,
                flag_ptr: Waddr,
                flag_size_ptr: Waddr,
            );
            sock_setsockopt(
                fd: Fd,
                level,
                name,
                flag_ptr: Waddr,
                flag_size: Size,
            );
        );
        pub const WASI_NAMES: [&str; 42 + 4 + 10] = add_wasi_names!(
            sock_listen, sock_bind,sock_connect, sock_open,
            sock_recv_from, sock_send_to, sock_getpeeraddr, sock_getlocaladdr,
            sock_getsockopt, sock_setsockopt
        );
    } else {
        declare_wasi_abis!(
            sock_accept(fd: Fd, flags, accepted_fd_ptr: Waddr);
        );
        pub const WASI_NAMES: [&str; 42 + 4] = add_wasi_names!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exists() {
        assert_eq!(args_get.name, "args_get");
        assert_eq!(args_get.args.len(), 2);
        assert_eq!(args_get.args[0].size, size_of::<i32>());
    }

    #[test]
    fn number() {
        cfg_if::cfg_if! {
            if #[cfg(feature = "wasmedge-sock")] {
                assert_eq!(WASI_NAMES.len(), 42 + 4 + 10);
                assert!(WASI_NAMES.iter().any(|&name| name == "sock_connect"));
            } else {
                assert_eq!(WASI_NAMES.len(), 42 + 4);
                const LAST_IDX:usize = 42 + 4 - 1;
                assert_eq!(WASI_NAMES[LAST_IDX], "sock_accept");
            }
        }
    }

    fn type_equals<A: 'static, B: 'static>(_a: &A, _b: &B) -> bool {
        core::any::TypeId::of::<A>() == core::any::TypeId::of::<B>()
    }

    #[test]
    fn param_type_0() {
        let _: sched_yield_params_t = ();
    }
    #[test]
    fn param_type_1() {
        let a0 = (0u32,);
        let a1 = (1i32,);
        let b: proc_exit_params_t = (1,);
        assert!(type_equals(&a0, &b));
        assert!(!type_equals(&a1, &b));
    }
    #[test]
    fn param_type_2() {
        let a0 = (0u32, 1u32);
        let a1 = (1i32, 2i32);
        let b: clock_res_get_params_t = (1, 2);
        assert!(!type_equals(&a0, &b));
        assert!(type_equals(&a1, &b));
    }
}
