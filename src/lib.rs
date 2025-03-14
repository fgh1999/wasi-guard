#![no_std]
#![feature(fn_traits)]
#![feature(tuple_trait)]

#[cfg(feature = "parse")]
pub mod abi;
pub mod policy;
pub mod util;
extern crate alloc;

pub use wasi;
pub use wasi_descriptor;

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn count_args_in_macro() {
        const SIZE0: usize = __count_idents!(a);
        assert_eq!(SIZE0, 1);
        const SIZE01: usize = __count_idents!(a, b, c);
        assert_eq!(SIZE01, 3);

        const SIZE10: usize = __count_idents!(a: i32);
        assert_eq!(SIZE10, 1);
        const SIZE11: usize = __count_idents!(a: i32, b: i32, c: i64, d: i8);
        assert_eq!(SIZE11, 4);

        const SIZE20: usize = __count_idents!(a[i32]);
        assert_eq!(SIZE20, 1);
        const SIZE21: usize = __count_idents!(a[i32], b[i32], c[i64]);
        assert_eq!(SIZE21, 3);

        const SIZE3: usize = __count_idents!(a, b: i64, c[8], d);
        assert_eq!(SIZE3, 4);
    }

    #[test]
    fn define_wasi_abi() {
        use wasi_descriptor::{desc_wasi_abi, DefaultAbiArgType};
        let wasi_args = desc_wasi_abi!(wasi_args(a, bb: i64, cc_c[8]));
        assert_eq!(wasi_args.name, "wasi_args");
        assert_eq!(wasi_args.args.len(), 3);
        assert_eq!(wasi_args.args[0].name, "a");
        assert_eq!(wasi_args.args[0].size, size_of::<DefaultAbiArgType>());
        assert_eq!(wasi_args.args[1].name, "bb");
        assert_eq!(wasi_args.args[1].size, size_of::<i64>());
        assert_eq!(wasi_args.args[2].name, "cc_c");
        assert_eq!(wasi_args.args[2].size, 8);
    }
}
