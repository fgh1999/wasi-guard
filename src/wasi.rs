pub type ArgSize = usize;
pub type DefaultAbiArgType = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiArg<'a> {
    pub name: &'a str,
    /// The size of the argument in bytes
    pub size: ArgSize,
}

macro_rules! desc_abi_arg {
    ($arg_name:ident [ $arg_size:expr ]) => {{
        const ARG_SIZE: $crate::wasi::ArgSize = $arg_size;
        $crate::wasi::AbiArg {
            name: stringify!($arg_name),
            size: ARG_SIZE,
        }
    }};
    ($arg_name:ident : $arg_type:ty) => {{
        $crate::wasi::AbiArg {
            name: stringify!($arg_name),
            size: core::mem::size_of::<$arg_type>(),
        }
    }};
    ($arg_name:ident) => {{
        $crate::wasi::AbiArg {
            name: stringify!($arg_name),
            size: core::mem::size_of::<$crate::wasi::DefaultAbiArgType>(),
        }
    }};
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiAbiDescriptor<'a, const ARG_NUM: usize> {
    pub name: &'a str,
    pub args: [AbiArg<'a>; ARG_NUM],
}
impl<'a, const ARG_NUM: usize> WasiAbiDescriptor<'a, ARG_NUM> {
    /// The return value of all WASI ABIs is always an i32(WASI Errno) .
    pub const fn ret_val_size() -> usize {
        core::mem::size_of::<i32>()
    }
    fn args_are_distinct(&self) -> bool {
        if self.args.is_empty() {
            return true;
        }
        self.args
            .iter()
            .all(|arg| self.args.iter().filter(|&a| a.name == arg.name).count() == 1)
    }
}

macro_rules! count_args {
    () => { 0usize };
    (, $($remain:tt)*) => { count_args!($($remain)*) }; // drop heading ','s
    ($head:ident) => { 1usize };
    ($head:ident, $($tail:tt)*) => { 1usize + count_args!($($tail)*) };
    ($head:ident : $head_ty:tt $($tail:tt)*) => { 1usize + count_args!($($tail)*) };
    ($head:ident [ $head_size:expr ] $($tail:tt)*) => { 1usize + count_args!($($tail)*) };
}

macro_rules! _desc_abi_arg_list {
    (@as_expr $e:expr) => {{$e}};
    (@accum () -> ($($body:tt)*)) => {_desc_abi_arg_list!(@as_expr [$($body)*])};
    // drop heading ','s in arg tts
    (@accum (, $($arg:tt)*) -> ($($body:tt)*)) => {
        _desc_abi_arg_list!(@accum ($($arg)*) -> ($($body)*))
    };
    // arg ...
    (@accum ($arg_name:ident) -> ($($body:tt)*)) => {
        _desc_abi_arg_list!(@accum () -> ($($body)* desc_abi_arg!($arg_name),))
    };
    (@accum ($arg_name:ident, $($tail:tt)*) -> ($($body:tt)*)) => {
        _desc_abi_arg_list!(@accum ($($tail)*) -> ($($body)* desc_abi_arg!($arg_name),))
    };
    // arg: type ...
    (@accum ($arg_name:ident:$arg_type:tt $($tail:tt)*) -> ($($body:tt)*)) => {
        _desc_abi_arg_list!(@accum ($($tail)*) -> ($($body)* desc_abi_arg!($arg_name:$arg_type),))
    };
    // arg[size] ...
    (@accum ($arg_name:ident[$arg_size:expr]  $($tail:tt)*) -> ($($body:tt)*)) => {
        _desc_abi_arg_list!(@accum ($($tail)*) -> ($($body)* desc_abi_arg!($arg_name[$arg_size]),))
    };
}
macro_rules! desc_abi_arg_list {
    [$($arg:tt)*] => {
        _desc_abi_arg_list!(@accum ($($arg)*) -> ())
    };
}

/// Macro to generate a WASI ABI descriptor.
#[macro_export]
macro_rules! desc_wasi_abi {
    ($wasi_name:ident) => {{
        WasiAbiDescriptor::<0> {
            name: stringify!($wasi_name),
            args: [],
        }
    }};
    ($wasi_name:ident ( $($arg:tt)* ) ) => {{
        const ARG_NUM: usize = count_args!($($arg)*);
        crate::wasi::WasiAbiDescriptor::<ARG_NUM> {
            name: stringify!($wasi_name),
            args: desc_abi_arg_list![$($arg)*],
        }
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn count_args_in_macro() {
        const SIZE0: usize = count_args!(a);
        assert_eq!(SIZE0, 1);
        const SIZE01: usize = count_args!(a, b, c);
        assert_eq!(SIZE01, 3);

        const SIZE10: usize = count_args!(a: i32);
        assert_eq!(SIZE10, 1);
        const SIZE11: usize = count_args!(a: i32, b: i32, c: i64, d: i8);
        assert_eq!(SIZE11, 4);

        const SIZE20: usize = count_args!(a[i32]);
        assert_eq!(SIZE20, 1);
        const SIZE21: usize = count_args!(a[i32], b[i32], c[i64]);
        assert_eq!(SIZE21, 3);

        const SIZE3: usize = count_args!(a, b: i64, c[8], d);
        assert_eq!(SIZE3, 4);
    }

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
}
