use wasi_guard_macros::all_tuples;

/// An extended Tuple trait of [`core::marker::Tuple`].
pub trait Tuple: core::marker::Tuple {
    const LENGTH: usize;
}
macro_rules! impl_tuple_length {
    ($($t:ident),*) => {
        impl<$($t),*> Tuple for ($($t,)*) {
            const LENGTH: usize = $crate::__count_idents!($($t),*);
        }
    };
}
all_tuples!(impl_tuple_length[0, 20]: T);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tuple_length() {
        assert_eq!(<() as Tuple>::LENGTH, 0);
        assert_eq!(<(i32,) as Tuple>::LENGTH, 1);
        struct Struct0;
        assert_eq!(<(i32, i64, Struct0) as Tuple>::LENGTH, 3);
    }
}
