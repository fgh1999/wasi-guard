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

pub trait PredicateFunction<'pred, Params: PredicateParams>: Sync + Send + 'pred {
    fn call(&self, params: Params) -> bool;
    // TODO: automatic param type conversion
}

macro_rules! impl_predicate_function_for_ptr {
    ($($Ptr_path:tt)+) => {
        impl<'pred, T, Params> PredicateFunction<'pred, Params> for $($Ptr_path)*<T>
        where
            T: PredicateFunction<'pred, Params>,
            Params: PredicateParams + 'pred,
        {
            fn call(&self, params: Params) -> bool {
                self.as_ref().call(params)
            }
        }
        impl<'pred, Params> PredicateFunction<'pred, Params> for $($Ptr_path)*<dyn PredicateFunction<'pred, Params>>
        where
            Params: PredicateParams + 'pred,
        {
            fn call(&self, params: Params) -> bool {
                self.as_ref().call(params)
            }
        }
    };
}
impl_predicate_function_for_ptr!(std::sync::Arc);

impl<'pred, T, Params> PredicateFunction<'pred, Params> for &'pred [T]
where
    T: PredicateFunction<'pred, Params>,
    Params: Clone + PredicateParams,
{
    fn call(&self, params: Params) -> bool {
        self.iter().all(|pred| pred.call(params.clone()))
    }
}
macro_rules! impl_predicate {
    ($($P:ident),*) => {
        impl<'pred, F, $($P,)*> $crate::policy::bound::PredicateFunction<'pred, ( $($P,)* )> for F
            where F: Sync + Send +::core::ops::Fn($($P,)*) -> bool + 'pred,
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

pub enum PredicateComposition<'pred, Params, A, B>
where
    Params: PredicateParams + Clone,
    A: PredicateFunction<'pred, Params>,
    B: PredicateFunction<'pred, Params>,
{
    And(A, B, PhantomData<(&'pred A, &'pred B, *const Params)>),
    Or(A, B, PhantomData<(&'pred A, &'pred B, *const Params)>),
}
unsafe impl<'pred, Params, A, B> Send for PredicateComposition<'pred, Params, A, B>
where
    Params: PredicateParams + Clone,
    A: PredicateFunction<'pred, Params>,
    B: PredicateFunction<'pred, Params>,
{
}
unsafe impl<'pred, Params, A, B> Sync for PredicateComposition<'pred, Params, A, B>
where
    Params: PredicateParams + Clone,
    A: PredicateFunction<'pred, Params>,
    B: PredicateFunction<'pred, Params>,
{
}
impl<'pred, Params, A, B> PredicateComposition<'pred, Params, A, B>
where
    Params: PredicateParams + Clone,
    A: PredicateFunction<'pred, Params>,
    B: PredicateFunction<'pred, Params>,
{
    pub fn all(a: A, b: B) -> Self {
        Self::And(a, b, PhantomData)
    }
    pub fn any(a: A, b: B) -> Self {
        Self::Or(a, b, PhantomData)
    }

    pub fn and<Other>(self, other: Other) -> PredicateComposition<'pred, Params, Self, Other>
    where
        Other: PredicateFunction<'pred, Params>,
    {
        PredicateComposition::all(self, other)
    }
    pub fn or<Other>(self, other: Other) -> PredicateComposition<'pred, Params, Self, Other>
    where
        Other: PredicateFunction<'pred, Params>,
    {
        PredicateComposition::any(self, other)
    }
}

impl<'pred, Params, A, B> PredicateFunction<'pred, Params>
    for PredicateComposition<'pred, Params, A, B>
where
    Params: PredicateParams + Clone,
    A: PredicateFunction<'pred, Params>,
    B: PredicateFunction<'pred, Params>,
    Self: 'pred,
{
    fn call(&self, params: Params) -> bool {
        match self {
            Self::And(a, b, _) => a.call(params.clone()) && b.call(params),
            Self::Or(a, b, _) => a.call(params.clone()) || b.call(params),
        }
    }
}

#[derive(Clone)]
pub struct AbiArgBound<'bound, Params: PredicateParams> {
    predicate: Arc<dyn PredicateFunction<'bound, Params>>,
}
// Safety: PredicateFunction<Params: PredicateParams>: Sync + Send
unsafe impl<'bound, Params: PredicateParams> Sync for AbiArgBound<'bound, Params> {}
unsafe impl<'bound, Params: PredicateParams> Send for AbiArgBound<'bound, Params> {}

impl<'bound, Params: PredicateParams> AbiArgBound<'bound, Params> {
    fn from_predicate(predicate: impl PredicateFunction<'bound, Params>) -> Self {
        Self {
            predicate: Arc::new(predicate),
        }
    }
    fn from_boxed_predicate(predicate: Box<dyn PredicateFunction<'bound, Params>>) -> Self {
        Self {
            predicate: predicate.into(),
        }
    }
}
impl<'bound, Params: PredicateParams + Clone + 'bound> AbiArgBound<'bound, Params> {
    pub fn and(self, other: Self) -> Self {
        let Self {
            predicate: this, ..
        } = self;
        let Self {
            predicate: other, ..
        } = other;
        Self::from_predicate(PredicateComposition::all(this, other))
    }
    pub fn or(self, other: Self) -> Self {
        let Self {
            predicate: this, ..
        } = self;
        let Self {
            predicate: other, ..
        } = other;
        Self::from_predicate(PredicateComposition::any(this, other))
    }
}

macro_rules! impl_from_fn_for_bound {
    ($($P:ident),*) => {
        impl<'bound, Predicate, $($P,)*> From<Predicate> for AbiArgBound<'bound, ( $($P,)* )>
        where ( $($P,)* ) : $crate::policy::bound::PredicateParams,
            Predicate : 'static + Fn($($P,)*) -> bool + Sync + Send,
        {
            fn from(predicate: Predicate) -> Self {
                Self::from_predicate(predicate)
            }
        }

        impl<'bound, $($P,)*> From<Box<dyn PredicateFunction<'bound, ($($P,)*)>>> for AbiArgBound<'bound, ( $($P,)* )>
        where ( $($P,)* ) : $crate::policy::bound::PredicateParams,
        {
            fn from(predicate: Box<dyn PredicateFunction<'bound, ($($P,)*)>>) -> Self {
                Self::from_boxed_predicate(predicate)
            }
        }
    };
}
all_tuples!(impl_from_fn_for_bound[0,10]: P);

macro_rules! impl_check_for_bound {
    ($($P:ident),*) => {
        impl<'bound, $($P,)*> AbiArgBound<'bound, ( $($P,)* )>
        where ( $($P,)* ) : $crate::policy::bound::PredicateParams + 'bound,
        {
            #[allow(non_snake_case, clippy::too_many_arguments)]
            pub fn check(&self, params: ( $($P,)* )) -> bool {
                // PredicateFunction::<'bound, ( $($P,)* )>::call(&self.predicate, params)
                self.predicate.call(params)
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
        let bound_0_0: AbiArgBound<()> = (|| true).into();
        assert!(bound_0_0.check(()));

        let bound_2_0: AbiArgBound<(i32, u64)> =
            Box::new(|a: i32, b: u64| -> bool { a > b as i32 }).into();
        assert!(bound_2_0.check((2, 1)));
        assert!(!bound_2_0.check((1, 2)));
    }

    #[test]
    fn fn_to_bound() {
        use crate::policy::bound::PredicateFunction;
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

        #[allow(dead_code)]
        struct Predicate1(i32);
        impl Predicate1 {
            #[allow(dead_code)]
            fn change(&mut self, n: i32) {
                self.0 = n;
            }
        }
        impl PredicateFunction<'static, (i32,)> for Predicate1 {
            fn call(&self, (a,): (i32,)) -> bool {
                a > 0
            }
        }
        // unsafe impl Sync for Predicate1 {}
        let boxed_predicate = Box::new(Predicate1(1));
        // TODO: why can't I use `into` here?
        // let bound_1_2: AbiArgBound<(i32,)> = boxed_predicate.into();
        let bound_1_2: AbiArgBound<(i32,)> = AbiArgBound::from_boxed_predicate(boxed_predicate);

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
        let bound_list = [|a: i32| a > 0, |a: i32| a < 233, |a: i32| a % 2 == 0];
        let bound_list: Vec<AbiArgBound<(i32,)>> =
            bound_list.into_iter().map(|bound| bound.into()).collect();
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

    #[test]
    fn bound_compositions() {
        use crate::policy::bound::AbiArgBound;

        let bound = [|a: i32| a < 0, |a: i32| a > 233, |a: i32| a % 2 == 0]
            .into_iter()
            .map(AbiArgBound::from_predicate)
            .fold(AbiArgBound::from_predicate(|_| false), |a, b| a.or(b));
        assert!(bound.check((100,)));
        assert!(!bound.check((223,)));
        assert!(bound.check((0,)));
        assert!(bound.check((-337,)));
        assert!(bound.check((299,)));

        let bounds: Vec<AbiArgBound<(i32,)>> =
            [|a: i32| a > 0, |a: i32| a < 233, |a: i32| a % 2 == 0]
                .into_iter()
                .map(|bound| bound.into())
                .collect();
        let bound = bounds
            .into_iter()
            .fold(AbiArgBound::from_predicate(|_| true), |a, b| a.and(b));
        assert!(bound.check((222,)));
        assert!(bound.check((100,)));
        assert!(!bound.check((111,)));
        assert!(!bound.check((-1,)));
        assert!(!bound.check((256,)));

        let local_bound = bound.clone();
        let closure = move || {
            fn ground_truth(x: i32) -> bool {
                x > 0 && x < 233 && x % 2 == 0
            }
            use rand::Rng;
            let mut rng = rand::rng();
            for x in (0..1000).map(|_| rng.random_range(-300..300)) {
                assert_eq!(ground_truth(x), local_bound.check((x,)));
            }
        };
        closure();

        let unstatic_bound = || {
            use rand::Rng;
            let mut rng = rand::rng();

            let (random_bounds, random_truth) = {
                let seq: Vec<_> = (0..10).map(|_| rng.random_range(-200..200)).collect();

                let bounds: Vec<_> = seq.iter().cloned().map(|x| move |a: i32| a != x).collect();

                let ground_truth = move |x: i32| seq.iter().all(|&y| x != y);
                (bounds, ground_truth)
            };

            let bounds: Vec<_> = random_bounds
                .into_iter()
                .map(AbiArgBound::from_predicate)
                .collect();
            let bound = bounds
                .into_iter()
                .fold(AbiArgBound::from_predicate(|_| true), |a, b| a.and(b));
            for x in (0..1000).map(|_| rng.random_range(-400..400)) {
                assert_eq!(random_truth(x), bound.check((x,)));
            }
        };
        unstatic_bound();
    }
}
