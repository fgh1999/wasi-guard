use core::fmt::Debug;
use std::{marker::PhantomData, sync::Arc};

use wasi_guard_macros::all_tuples;

use crate::util::Tuple;

pub trait PredicateParam: Sized + Debug {}
pub trait PredicateParams: Tuple + Debug {}
macro_rules! impl_predicate_param_for_tuple {
    ($($P:ident),*) => {
        impl<$($P),*> PredicateParams for ($($P,)*)
        where $( $P : PredicateParam, )*
        {}
    };
}
all_tuples!(impl_predicate_param_for_tuple[0, 10]: P);

macro_rules! impl_predicate_param {
    ($type:ty) => {
        impl PredicateParam for $type {}
    };
    ($type:ty $(, $tail:ty)*) => {
        impl_predicate_param!($type);
        impl_predicate_param!($($tail),*);
    };
}
impl_predicate_param!(bool, i32, u32, i64, u64, f32, f64);

pub trait PredicateFunction<Params: PredicateParams> {
    fn call(&self, params: Params) -> bool;
}
macro_rules! impl_predicate_function_for_ptr {
    ($($Ptr_path:tt)+) => {
        impl<T, Params> PredicateFunction<Params> for $($Ptr_path)*<T>
        where
            T: PredicateFunction<Params>,
            Params: PredicateParams,
        {
            fn call(&self, params: Params) -> bool {
                self.as_ref().call(params)
            }
        }
        impl<Params> PredicateFunction<Params> for $($Ptr_path)*<dyn PredicateFunction<Params>>
        where
            Params: PredicateParams,
        {
            fn call(&self, params: Params) -> bool {
                self.as_ref().call(params)
            }
        }
    };
}
impl_predicate_function_for_ptr!(std::sync::Arc);
impl_predicate_function_for_ptr!(std::rc::Rc);
impl<'a, T, Params> PredicateFunction<Params> for &'a [T]
where
    T: PredicateFunction<Params>,
    Params: Clone + PredicateParams,
{
    fn call(&self, params: Params) -> bool {
        self.iter().all(|pred| pred.call(params.clone()))
    }
}
macro_rules! impl_predicate {
    ($($P:ident),*) => {
        impl<F, $($P,)*> $crate::policy::bound::PredicateFunction<( $($P,)* )> for F
            where F: ::core::ops::Fn($($P,)*) -> bool,
                ( $($P,)* ) : $crate::policy::bound::PredicateParams,
        {
            #[allow(non_snake_case, clippy::too_many_arguments)]
            fn call(&self, ($($P,)*): ( $($P,)* )) -> bool {
                self($($P),*)
            }
        }
    };
}
all_tuples!(impl_predicate[0, 10]: P);
// TODO: impl compositions of [`PredicateFunction<Params>`]: `.and(..)` and `.or(..)`.

pub enum PredicateComposition<Params, A, B>
where
    Params: PredicateParams,
    A: PredicateFunction<Params>,
    B: PredicateFunction<Params>,
{
    And(A, B, PhantomData<Arc<dyn PredicateFunction<Params>>>),
    Or(A, B, PhantomData<Arc<dyn PredicateFunction<Params>>>),
}
impl<Params, A, B> PredicateComposition<Params, A, B>
where
    Params: PredicateParams,
    A: PredicateFunction<Params>,
    B: PredicateFunction<Params>,
{
    pub fn all(a: A, b: B) -> Self {
        Self::And(a, b, PhantomData)
    }
    pub fn any(a: A, b: B) -> Self {
        Self::Or(a, b, PhantomData)
    }

    pub fn and<Other>(self, other: Other) -> PredicateComposition<Params, Self, Other>
    where
        Other: PredicateFunction<Params>,
        Params: Clone,
    {
        <Self as CompositePredicate<Params, Other>>::and(self, other)
    }
    pub fn or<Other>(self, other: Other) -> PredicateComposition<Params, Self, Other>
    where
        Other: PredicateFunction<Params>,
        Params: Clone,
    {
        <Self as CompositePredicate<Params, Other>>::or(self, other)
    }
}

impl<Params, A, B> PredicateFunction<Params> for PredicateComposition<Params, A, B>
where
    Params: PredicateParams + Clone,
    A: PredicateFunction<Params>,
    B: PredicateFunction<Params>,
{
    fn call(&self, params: Params) -> bool {
        match self {
            Self::And(a, b, _) => a.call(params.clone()) && b.call(params),
            Self::Or(a, b, _) => a.call(params.clone()) || b.call(params),
        }
    }
}

trait CompositePredicate<Params: PredicateParams + Clone, Other>
where
    Self: Sized + PredicateFunction<Params>,
    Other: PredicateFunction<Params>,
{
    fn and(self, other: Other) -> PredicateComposition<Params, Self, Other>;
    fn or(self, other: Other) -> PredicateComposition<Params, Self, Other>;
}
impl<Params, A, B> CompositePredicate<Params, B> for A
where
    Params: PredicateParams + Clone,
    A: PredicateFunction<Params>,
    B: PredicateFunction<Params>,
{
    fn and(self, other: B) -> PredicateComposition<Params, Self, B> {
        PredicateComposition::all(self, other)
    }
    fn or(self, other: B) -> PredicateComposition<Params, Self, B> {
        PredicateComposition::any(self, other)
    }
}
// macro_rules! impl_composite_predicate {
//     ($($P:ident),*) => {
//         impl<F, $($P,)* Other> CompositePredicate<( $($P,)* ), Other> for F
//             where F: ::std::ops::Fn($($P,)*) -> bool,
//                 ( $($P,)* ) : $crate::policy::bound::PredicateParams,
//                 ( $($P,)* ) : ::core::clone::Clone,
//                 Other: $crate::policy::bound::PredicateFunction<( $($P,)* )>,
//         {
//             fn and(self, other: Other) -> PredicateComposition<( $($P,)* ), Self, Other> {
//                 PredicateComposition::all(self, other)
//             }
//             fn or(self, other: Other) -> PredicateComposition<( $($P,)* ), Self, Other> {
//                 PredicateComposition::any(self, other)
//             }
//         }
//     };
// }
// all_tuples!(impl_composite_predicate[0, 10]: P);

#[derive(Clone)]
pub struct AbiArgBound<Params: PredicateParams> {
    predicate: Arc<dyn PredicateFunction<Params>>,
}
impl<Params: PredicateParams> AbiArgBound<Params> {
    fn from_predicate(predicate: impl PredicateFunction<Params> + 'static) -> Self {
        Self {
            predicate: Arc::new(predicate),
        }
    }
    fn from_boxed_predicate(predicate: Box<dyn PredicateFunction<Params>>) -> Self {
        Self {
            predicate: Arc::from(predicate),
        }
    }
}
// impl<Params: PredicateParams + Clone> AbiArgBound<Params> {
//     pub fn and<Other>(self, other: Other) -> Self
//     where
//         Other: PredicateFunction<Params>,
//         Params: Clone,
//     {
//         let Self { predicate, ..} = self;
//         let predicate = PredicateComposition::all(predicate, other);
//         Self::from_predicate(predicate)
//     }
//     pub fn or<Other>(self, other: Other) -> Self
//     where
//         Other: PredicateFunction<Params>,
//         Params: Clone,
//     {
//         todo!()
//         // <Self as CompositePredicate<Params, Other>>::or(self, other)
//     }
// }

macro_rules! impl_from_fn_for_bound {
    ($($P:ident),*) => {
        impl<Predicate, $($P,)*> From<Predicate> for AbiArgBound<( $($P,)* )>
        where ( $($P,)* ) : $crate::policy::bound::PredicateParams,
            Predicate : Fn($($P,)*) -> bool + 'static,
        {
            fn from(predicate: Predicate) -> Self {
                Self::from_predicate(predicate)
            }
        }

        impl<$($P,)*> From<Box<dyn PredicateFunction<($($P,)*)>>> for AbiArgBound<( $($P,)* )>
        where ( $($P,)* ) : $crate::policy::bound::PredicateParams,
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
        where ( $($P,)* ) : $crate::policy::bound::PredicateParams,
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
        let bound_0_0: AbiArgBound<()> = (|| true).into();
        assert!(bound_0_0.check(()));

        let bound_2_0: AbiArgBound<(i32, u64)> =
            Box::new(|a: i32, b: u64| -> bool { a > b as i32 }).into();
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

    #[test]
    fn predicate_compositions() {
        use crate::policy::bound::{PredicateComposition, PredicateFunction};

        let pred_0_0 = || true;
        let pred_0_1 = || false;
        let pred_0_2 = PredicateComposition::all(pred_0_0, pred_0_1);
        assert!(!pred_0_2.call(()));
        let pred_0_3 = PredicateComposition::any(pred_0_0, pred_0_1);
        assert!(pred_0_3.call(()));

        let pred_1_0 = |a: i32| a > 0;
        let pred_1_1 = |b: i32| b < 10;
        let pred_1_2 = |x: i32| x > 0 && x < 10;
        // homomorphism
        let pred_1_3 = PredicateComposition::all(pred_1_0, pred_1_1);
        for i in -20..20 {
            assert_eq!(pred_1_2(i), pred_1_3.call((i,)));
        }
        let pred_1_4 = |x: i32| x > 0 || x < 10;
        let pred_1_5 = PredicateComposition::any(pred_1_0, pred_1_1);
        for i in -20..20 {
            assert_eq!(pred_1_4(i,), pred_1_5.call((i,)));
        }

        let pred_2_0 = |a: i32, b: u64| a < b as i32;
        let pred_2_1 = |a: i32, b: u64| a > b as i32;
        let pred_2_2 = PredicateComposition::all(pred_2_0, pred_2_1);
        assert!(!pred_2_2.call((-1, 1)));
        assert!(!pred_2_2.call((0, 0)));
    }
}
