use wasi_guard_macros::all_tuples;

#[test]
fn parse_all_tuples() {
    struct A;

    trait Feature0<Params> {
        fn foo(params: Params) -> Params;
    }
    macro_rules! mr0 {
        ($($P:ident),*) => {
            impl<$($P),*> Feature0<($($P,)*)> for A {
                fn foo(params: ($($P,)*)) -> ($($P,)*) {
                    params
                }
            }
        };
    }
    all_tuples!(mr0[0, 3]: P);
    assert_eq!(A::foo(()), ());
    assert_eq!(A::foo((0,)), (0,));
    assert_eq!(A::foo((0, 1i64)), (0, 1i64));
    assert_eq!(A::foo((-1, 1i64, 3.14f32)), (-1, 1i64, 3.14f32));

    trait Feature1<Params> {
        fn bar(params: Params) -> Params;
    }
    macro_rules! mr1 {
        ($($P:ident),*) => {
            impl<$($P),*> Feature1<($($P,)*)> for A {
                fn bar(params: ($($P,)*)) -> ($($P,)*) {
                    params
                }
            }
        };
    }
    all_tuples!(mr1[0, 3]: SomeTypeName233);
    assert_eq!(A::bar(()), ());
    assert_eq!(A::bar((0,)), (0,));
    assert_eq!(A::bar((0, 1i64)), (0, 1i64));
    assert_eq!(A::bar((-1, 1i64, 3.14f32)), (-1, 1i64, 3.14f32));
}

#[test]
fn count_types_in_tuple() {
    trait TupleLength {
        const LENGTH: usize;
    }
    macro_rules! __count_idents {
        () => { 0usize };
        (, $($remain:tt)*) => { __count_idents!($($remain)*) }; // drop heading ','s
        ($head:ident) => { 1usize };
        ($head:ident, $($tail:tt)*) => { 1usize + __count_idents!($($tail)*) };
        ($head:ident : $head_ty:tt $($tail:tt)*) => { 1usize + __count_idents!($($tail)*) };
        ($head:ident [ $head_size:expr ] $($tail:tt)*) => { 1usize + __count_idents!($($tail)*) };
    }
    macro_rules! impl_tuple_length {
        ($($t:ident),*) => {
            impl<$($t),*> TupleLength for ($($t,)*) {
                const LENGTH: usize = __count_idents!($($t),*);
            }
        };
    }
    all_tuples!(impl_tuple_length[0, 3 ]: T);

    assert_eq!(<()>::LENGTH, 0);
    assert_eq!(<(i32,)>::LENGTH, 1);
    assert_eq!(<(u64, bool)>::LENGTH, 2);
    struct A;
    assert_eq!(<(u64, bool, A)>::LENGTH, 3);
}
