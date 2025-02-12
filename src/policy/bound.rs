use wasi_guard_macros::all_tuples;

pub trait PredicateParam: Sized {}

macro_rules! impl_predicate_param {
    ($type:ty) => {
        impl PredicateParam for $type {}
    };
    ($type:ty $(, $tail:ty)*) => {
        impl_predicate_param!($type);
        impl_predicate_param!($($tail),*);
    };
}
impl_predicate_param!(i32, u32, i64, u64, f32, f64);

pub trait PredicateFunction<Params> {
    fn call(&self, params: Params) -> bool;
}
impl<Params> PredicateFunction<Params> for Box<dyn PredicateFunction<Params>> {
    fn call(&self, params: Params) -> bool {
        self.as_ref().call(params)
    }
}
impl<'a, T, Params> PredicateFunction<Params> for &'a [T]
where
    T: PredicateFunction<Params>,
    Params: Clone,
{
    fn call(&self, params: Params) -> bool {
        self.iter().all(|pred| pred.call(params.clone()))
    }
}
// TODO: 用函数式过程宏定义all_tuples!
macro_rules! impl_predicate {
    ($($P:ident),*) => {
        impl<F, $($P,)*> $crate::policy::bound::PredicateFunction<( $($P,)* )> for F
            where F: ::std::ops::Fn($($P,)*) -> bool,
                  $( $P : $crate::policy::bound::PredicateParam, )*
        {
            #[allow(non_snake_case, clippy::too_many_arguments)]
            fn call(&self, ($($P,)*): ( $($P,)* )) -> bool {
                self($($P,)*)
            }
        }
    };
}
all_tuples!(impl_predicate[0, 10]: P);
// TODO: impl compositions of [`PredicateFunction<Params>`]: `.and(..)` and `.or(..)`.

pub struct AbiArgBound<Params>
where
    Params: core::fmt::Debug,
{
    predicate: Box<dyn PredicateFunction<Params>>,
    _phantom: core::marker::PhantomData<Params>,
}
impl<Params> AbiArgBound<Params>
where
    Params: core::fmt::Debug,
{
    fn from_predicate(predicate: impl PredicateFunction<Params> + 'static) -> Self {
        Self {
            predicate: Box::new(predicate),
            _phantom: core::marker::PhantomData,
        }
    }
    fn from_boxed_predicate(predicate: Box<dyn PredicateFunction<Params>>) -> Self {
        Self {
            predicate,
            _phantom: core::marker::PhantomData,
        }
    }
}

macro_rules! impl_from_fn_for_bound {
    ($($P:ident),*) => {
        impl<Predicate, $($P,)*> From<Predicate> for AbiArgBound<( $($P,)* )>
        where $( $P : PredicateParam, )* $( $P : core::fmt::Debug, )*
            Predicate : Fn($($P,)*) -> bool + 'static,
        {
            fn from(predicate: Predicate) -> Self {
                Self::from_predicate(predicate)
            }
        }

        impl<$($P,)*> From<Box<dyn PredicateFunction<($($P,)*)>>> for AbiArgBound<( $($P,)* )>
        where $( $P : PredicateParam, )* $( $P : core::fmt::Debug, )*
        {
            fn from(predicate: Box<dyn PredicateFunction<($($P,)*)>>) -> Self {
                Self::from_boxed_predicate(predicate)
            }
        }
    };
}
all_tuples!(impl_from_fn_for_bound[0,10]: P);

pub trait CheckArgBound {
    type Params;
    fn check(&self, params: Self::Params) -> bool;
}

macro_rules! impl_check_for_bound {
    ($($P:ident),*) => {
        impl<$($P,)*> crate::policy::bound::CheckArgBound for AbiArgBound<( $($P,)* )>
        where $( $P : $crate::policy::bound::PredicateParam, )*
            $( $P : core::fmt::Debug, )*
        {
            type Params = ( $($P,)* );
            #[allow(non_snake_case, clippy::too_many_arguments)]
            fn check(&self, params: Self::Params) -> bool {
                PredicateFunction::<Self::Params>::call(self.predicate.as_ref(), params)
            }
        }
    };
}
all_tuples!(impl_check_for_bound[0,10]: P);

#[cfg(test)]
mod test {
    use super::AbiArgBound;

    #[test]
    fn predicate_on_closure() {
        const F_0_0: fn() -> i32 = || -> i32 { 233 };
        assert_eq!(F_0_0.call(()), F_0_0());
        assert_eq!(F_0_0.call(()), 233);

        const F_2_0: fn(i32, u64) -> bool = |a: i32, b: u64| -> bool { a - b as i32 > 0 };
        assert!(F_2_0.call((2, 1)));
        assert_eq!(F_2_0(2323, 233), F_2_0.call((2323, 233)));

        let f_1_0 = |flag: bool| flag;
        assert!(f_1_0(true) && !f_1_0(false));

        let f_2_0 = |a: i32, b: u64| -> bool { a - b as i32 > 0 };
        assert!(f_2_0.call((2, 1)));
        assert_eq!(f_2_0(2323, 233), f_2_0.call((2323, 233)));
    }

    #[test]
    fn bound_on_closure() {
        use crate::policy::bound::CheckArgBound;
        let bound_0_0 = AbiArgBound {
            predicate: Box::new(|| true),
            _phantom: core::marker::PhantomData,
        };
        assert!(bound_0_0.check(()));

        let bound_2_0 = AbiArgBound::<(i32, u64)> {
            predicate: Box::new(|a: i32, b: u64| -> bool { a > b as i32 }),
            _phantom: core::marker::PhantomData::<(i32, u64)>,
        };
        assert!(bound_2_0.check((2, 1)));
        assert!(!bound_2_0.check((1, 2)));
    }

    #[test]
    fn fn_to_bound() {
        use crate::policy::bound::{CheckArgBound, PredicateFunction};
        fn predicate_0(a: i32) -> bool {
            a > 0
        }
        let bound_0: AbiArgBound<(i32,)> = predicate_0.into();
        assert!(bound_0.check((1,)));
        assert!(!bound_0.check((-1,)));

        let predicate_1 = |a: i32| a > 0;
        let bound_1_0: AbiArgBound<(i32,)> = predicate_1.into();
        assert!(bound_1_0.check((1,)));
        assert!(!bound_1_0.check((-1,)));
        let predicate_1 = Box::new(predicate_1);
        let bound_1_1: AbiArgBound<(i32,)> = predicate_1.into();
        assert!(bound_1_1.check((1,)));
        assert!(!bound_1_1.check((-1,)));

        struct Predicate1;
        impl PredicateFunction<(i32,)> for Predicate1 {
            fn call(&self, (a,): (i32,)) -> bool {
                a > 0
            }
        }
        let boxed_predicate: Box<dyn PredicateFunction<(i32,)>> = Box::new(Predicate1);
        let bound_1_2: AbiArgBound<(i32,)> = boxed_predicate.into();
        assert!(bound_1_2.check((1,)));
        assert!(!bound_1_2.check((-1,)));

        fn predicate_2(a: i32, b: u64) -> bool {
            a < b as i32
        }
        let bound_2: AbiArgBound<(i32, u64)> = predicate_2.into();
        assert!(bound_2.check((-1, 233)));
        assert!(!bound_2.check((233, 100)));
    }

    #[test]
    fn bound_list() {
        use crate::policy::bound::CheckArgBound;
        let bound_list = [|a: i32| a > 0, |a: i32| a < 233, |a: i32| a % 2 == 0];
        let bound_list: Vec<AbiArgBound<(i32,)>> =
            bound_list.iter().map(|&bound| bound.into()).collect();
        assert!(bound_list.iter().all(|bound| bound.check((222,))));
        assert!(!bound_list.iter().all(|bound| bound.check((111,))));
    }
}
