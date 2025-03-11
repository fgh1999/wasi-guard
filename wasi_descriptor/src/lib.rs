#![no_std]

pub type ArgSize = usize;
pub type DefaultAbiArgType = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiArg<'a> {
    pub name: &'a str,
    /// The size of the argument in bytes
    pub size: ArgSize,
}

/// Define an [`AbiArg`].
#[macro_export]
macro_rules! desc_abi_arg {
    ($arg_name:ident [ $arg_size:expr ]) => {{
        const ARG_SIZE: $crate::ArgSize = $arg_size;
        $crate::AbiArg {
            name: stringify!($arg_name),
            size: ARG_SIZE,
        }
    }};
    ($arg_name:ident : $arg_type:ty) => {{
        $crate::AbiArg {
            name: stringify!($arg_name),
            size: core::mem::size_of::<$arg_type>(),
        }
    }};
    ($arg_name:ident) => {{
        $crate::AbiArg {
            name: stringify!($arg_name),
            size: core::mem::size_of::<$crate::DefaultAbiArgType>(),
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
    pub fn args_are_distinct(&self) -> bool {
        if self.args.is_empty() {
            return true;
        }
        self.args
            .iter()
            .all(|arg| self.args.iter().filter(|&a| a.name == arg.name).count() == 1)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! _desc_abi_arg_list {
    (@as_expr $e:expr) => {{$e}};
    (@accum () -> ($($body:tt)*)) => {$crate::_desc_abi_arg_list!(@as_expr [$($body)*])};
    // drop heading ','s in arg tts
    (@accum (, $($arg:tt)*) -> ($($body:tt)*)) => {
        $crate::_desc_abi_arg_list!(@accum ($($arg)*) -> ($($body)*))
    };
    // arg ...
    (@accum ($arg_name:ident) -> ($($body:tt)*)) => {
        $crate::_desc_abi_arg_list!(@accum () -> ($($body)* $crate::desc_abi_arg!($arg_name),))
    };
    (@accum ($arg_name:ident, $($tail:tt)*) -> ($($body:tt)*)) => {
        $crate::_desc_abi_arg_list!(@accum ($($tail)*) -> ($($body)* $crate::desc_abi_arg!($arg_name),))
    };
    // arg: type ...
    (@accum ($arg_name:ident:$arg_type:tt $($tail:tt)*) -> ($($body:tt)*)) => {
        $crate::_desc_abi_arg_list!(@accum ($($tail)*) -> ($($body)* $crate::desc_abi_arg!($arg_name:$arg_type),))
    };
    // arg[size] ...
    (@accum ($arg_name:ident[$arg_size:expr]  $($tail:tt)*) -> ($($body:tt)*)) => {
        $crate::_desc_abi_arg_list!(@accum ($($tail)*) -> ($($body)* $crate::desc_abi_arg!($arg_name[$arg_size]),))
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __count_idents {
    () => { 0usize };
    (, $($remain:tt)*) => { $crate::__count_idents!($($remain)*) }; // drop heading ','s
    ($head:ident) => { 1usize };
    ($head:ident, $($tail:tt)*) => { 1usize + $crate::__count_idents!($($tail)*) };
    ($head:ident : $head_ty:tt $($tail:tt)*) => { 1usize + $crate::__count_idents!($($tail)*) };
    ($head:ident [ $head_size:expr ] $($tail:tt)*) => { 1usize + $crate::__count_idents!($($tail)*) };
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
        $crate::WasiAbiDescriptor::<{$crate::__count_idents!($($arg)*)}> {
            name: stringify!($wasi_name),
            args: $crate::_desc_abi_arg_list!(@accum ($($arg)*) -> ()),
        }
    }};
}
