use wasi_guard::{
    desc_wasi_abi,
    policy::{action::Action, bound::AbiArgBound},
};

#[test]
fn full_statement() {
    let wasi = desc_wasi_abi!(clock_time_get(clock_id, precision[8]));
    let statement = wasi.trigger(Action::Allow);

    let bound = |a: i32, b: u64| -> bool { a > 0 && b <= 1 << 8 };
    let statement = statement.when(AbiArgBound::from(bound));
    assert!(statement.check_bound((1, 233)));
    assert!(!statement.check_bound((0, 1 << 9)));
}
// claim a statement:
// action { $(abi $(where bound)?),+ }

// macro_rules! statement {
//     {$action:ident { $($abi:ident $(where $bound:expr)?),+ } } => {

//     };
// }

// policy! {
//     some_action! {
//         $(abi $(where $(bound)*)?),*
//     }
// }

// macro_rules! policy {
//     [$($type:ty),*] => {
//         struct Policy_($($type),*);
//     };
//     {$($type:ty),*} => {
//         ($($type,)*)
//     };
// }

// policy! [i32, u64];

// #[test]
// fn test_macro() {
//     let _a: Policy_ = Policy_(1, 2);
//     // println!{"{:?}", _a};
//     type Policy = policy!{i32, i32};
// }
