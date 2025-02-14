use wasi_guard_macros::all_tuples;

pub(crate) trait TupleLength {
    const TYPE_LENGTH: usize;
}
macro_rules! impl_tuple_length {
    ($($t:ident),*) => {
        impl<$($t),*> TupleLength for ($($t,)*) {
            const TYPE_LENGTH: usize = $crate::__count_idents!($($t),*);
        }
    };
}
all_tuples!(impl_tuple_length[0, 20]: T);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_tuple_length() {
        assert_eq!(<() as TupleLength>::TYPE_LENGTH, 0);
        assert_eq!(<(i32,) as TupleLength>::TYPE_LENGTH, 1);
        struct Struct0;
        assert_eq!(<(i32, i64, Struct0) as TupleLength>::TYPE_LENGTH, 3);
    }
}
