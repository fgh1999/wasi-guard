#![no_std]

pub mod p1;
#[cfg(feature = "p1")]
pub use p1::*;

pub mod p2 {
    // TODO
}
#[cfg(feature = "p2")]
pub use p2::*;

#[doc(hidden)]
#[macro_export]
macro_rules! __count_args {
    () => { 0usize };
    (, $($remain:tt)*) => { $crate::__count_args!($($remain)*) }; // drop heading ','s
    ($head:ident) => { 1usize };
    ($head:ident, $($tail:tt)*) => { 1usize + $crate::__count_args!($($tail)*) };
    ($head:ident : $head_ty:tt $($tail:tt)*) => { 1usize + $crate::__count_args!($($tail)*) };
    ($head:ident [ $head_size:expr ] $($tail:tt)*) => { 1usize + $crate::__count_args!($($tail)*) };
}
#[macro_export]
macro_rules! type_from_arg {
    ($arg:ident) => {
        wasi_descriptor::DefaultAbiArgType
    };
    ($arg:ident : $ty:ty) => {
        $ty
    };
    ($arg:ident [ $size:expr ]) => {
        [u8; $size]
    };
}
#[macro_export]
macro_rules! _param_type_from_args {
    (@expr $($expr:tt)*) => { ( $($expr)* ) };
    (@accum () -> ( $($res:tt)* )) => {
        $crate::_param_type_from_args!(@expr $($res)* )
    };
    // drop heading commas in accum input
    (@accum (, $($rest:tt)*) -> ( $($res:tt)* )) => {
        $crate::_param_type_from_args!(@accum ( $($rest)* ) -> ( $($res)* ))
    };

    (@accum ($arg:ident : $ty:ty) -> ( $($res:tt)* )) => {
        $crate::_param_type_from_args!(@accum () -> ( $($res)* $crate::type_from_arg!($arg:$ty), ))
    };
    (@accum ($arg:ident : $ty:ty, $($rest:tt)*) -> ( $($res:tt)* )) => {
        $crate::_param_type_from_args!(@accum ($($rest)*) -> ( $($res)* $crate::type_from_arg!($arg:$ty), ))
    };
    (@accum ($arg:ident[ $size:expr ]  $($rest:tt)*) -> ( $($res:tt)* )) => {
        $crate::_param_type_from_args!(@accum ($($rest)*) -> ( $($res)* $crate::type_from_arg!($arg[$size]), ))
    };
    (@accum ($arg:ident) -> ( $($res:tt)* )) => {
        $crate::_param_type_from_args!(@accum () -> ( $($res)* $crate::type_from_arg!($arg), ))
    };
    (@accum ($arg:ident, $($rest:tt)*) -> ( $($res:tt)* )) => {
        $crate::_param_type_from_args!(@accum ($($rest)*) -> ( $($res)* $crate::type_from_arg!($arg), ))
    };
}
#[macro_export]
macro_rules! param_type_from_args {
    ($($token:tt)*) => {
        $crate::_param_type_from_args!(@accum ($($token)*) -> ())
    };
}

#[macro_export]
macro_rules! _default_param_type_from_args {
    (@expr $($expr:tt)*) => { ( $($expr)* ) };
    (@accum () -> ( $($res:tt)* )) => {
        $crate::_default_param_type_from_args!(@expr $($res)* )
    };
    // drop heading commas in accum input
    (@accum (, $($rest:tt)*) -> ( $($res:tt)* )) => {
        $crate::_default_param_type_from_args!(@accum ( $($rest)* ) -> ( $($res)* ))
    };

    (@accum ($arg:ident : $ty:ty) -> ( $($res:tt)* )) => {
        $crate::_default_param_type_from_args!(@accum () -> ( $($res)* $crate::type_from_arg!($arg), ))
    };
    (@accum ($arg:ident : $ty:ty, $($rest:tt)*) -> ( $($res:tt)* )) => {
        $crate::_default_param_type_from_args!(@accum ($($rest)*) -> ( $($res)* $crate::type_from_arg!($arg), ))
    };
    (@accum ($arg:ident[ $size:expr ]  $($rest:tt)*) -> ( $($res:tt)* )) => {
        $crate::_default_param_type_from_args!(@accum ($($rest)*) -> ( $($res)* $crate::type_from_arg!($arg), ))
    };
    (@accum ($arg:ident) -> ( $($res:tt)* )) => {
        $crate::_default_param_type_from_args!(@accum () -> ( $($res)* $crate::type_from_arg!($arg), ))
    };
    (@accum ($arg:ident, $($rest:tt)*) -> ( $($res:tt)* )) => {
        $crate::_default_param_type_from_args!(@accum ($($rest)*) -> ( $($res)* $crate::type_from_arg!($arg), ))
    };
}
#[macro_export]
macro_rules! default_param_type_from_args {
    ($($token:tt)*) => {
        $crate::_default_param_type_from_args!(@accum ($($token)*) -> ())
    };
}

#[macro_export]
macro_rules! declare_wasi_abis {
    () => {};
    (; $( $rest:tt )*) => {
        $crate::declare_wasi_abis!($($rest)*);
    };
    ($wasi_name:ident $(( $($arg:tt)* ))? ) => {
        paste::paste! {
            #[allow(non_camel_case_types)]
            pub type [<$wasi_name _params_t>] = $crate::param_type_from_args!($($($arg)*)*);
            #[allow(non_camel_case_types)]
            pub type [<$wasi_name _params_default_t>] = $crate::default_param_type_from_args!($($($arg)*)*);
        }
        #[allow(non_upper_case_globals)]
        pub const $wasi_name: wasi_descriptor::WasiAbiDescriptor<{$crate::__count_args!($($($arg)*)*)}> = wasi_descriptor::desc_wasi_abi!($wasi_name $(($($arg)*))*);
    };
    ($wasi_name:ident $(( $($arg:tt)* ))? ; $( $rest:tt )*) => {
        $crate::declare_wasi_abis!($wasi_name $(( $($arg)* ))?);
        $crate::declare_wasi_abis!($($rest)*);
    };
}
