use wasi_descriptor::{desc_abi_arg, desc_wasi_abi, AbiArg, DefaultAbiArgType, WasiAbiDescriptor};

#[test]
fn abi_arg_from_macro() {
    const ARG0: AbiArg = desc_abi_arg!(a);
    assert_eq!(ARG0.name, "a");
    assert_eq!(ARG0.size, size_of::<DefaultAbiArgType>());
    const ARG1: AbiArg = desc_abi_arg!(b: i64);
    assert_eq!(ARG1.name, "b");
    assert_eq!(ARG1.size, size_of::<i64>());
    const ARG2: AbiArg = desc_abi_arg!(b[2]);
    assert_eq!(ARG2.name, "b");
    assert_eq!(ARG2.size, 2);

    const CAMEL_CASE: AbiArg = desc_abi_arg!(camelCase[8]);
    assert_eq!(CAMEL_CASE.name, "camelCase");
    assert_eq!(CAMEL_CASE.size, 8);
    const SNAKE_CASE: AbiArg = desc_abi_arg!(snake_case);
    assert_eq!(SNAKE_CASE.name, "snake_case");
    assert_eq!(SNAKE_CASE.size, size_of::<DefaultAbiArgType>());
}

const A: WasiAbiDescriptor<0> = desc_wasi_abi!(A);
const A1: WasiAbiDescriptor<0> = desc_wasi_abi!(A1());
const B: WasiAbiDescriptor<1> = desc_wasi_abi!(proc_exit(code));
const B1: WasiAbiDescriptor<1> = desc_wasi_abi!(proc_exit(code,));
const C: WasiAbiDescriptor<2> = desc_wasi_abi!(clock_time_get(clock_id, precision[8]));
const D: WasiAbiDescriptor<1> = desc_wasi_abi!(close(fd: u64));
const E: WasiAbiDescriptor<4> = desc_wasi_abi!(wasi(arg_0[8], arg1, ARG2: i64, arg3));

#[test]
fn args_num() {
    assert_eq!(A.args.len(), 0);
    assert_eq!(A1.args.len(), 0);
    assert_eq!(B.args.len(), 1);
    assert_eq!(B1.args.len(), 1);
    assert_eq!(C.args.len(), 2);
    assert_eq!(D.args.len(), 1);
    assert_eq!(E.args.len(), 4);

    let wasi_abi = desc_wasi_abi!(wasi_abi(arg0, arg1, arg2));
    assert_eq!(wasi_abi.args.len(), 3);
}

#[test]
fn arg_size() {
    assert_eq!(B1.args[0].size, size_of::<DefaultAbiArgType>());
    assert_eq!(C.args[0].size, size_of::<DefaultAbiArgType>());
    assert_eq!(C.args[1].size, 8);
    assert_eq!(D.args[0].size, size_of::<u64>());
    assert_eq!(E.args[0].size, 8);
    assert_eq!(E.args[1].size, size_of::<DefaultAbiArgType>());
    assert_eq!(E.args[2].size, size_of::<i64>());
    assert_eq!(E.args[3].size, size_of::<DefaultAbiArgType>());
}

#[test]
fn distinct_arg_names() {
    assert!(E.args_are_distinct());

    let wasi_abi = desc_wasi_abi!(wasi_abi(arg0, arg1, arg0));
    assert!(!wasi_abi.args_are_distinct());
}
