#![feature(fn_traits)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod abi;
#[macro_use]
mod wasi;
mod policy;
mod util;

use std::{collections::HashMap, rc::Rc};

use abi::ImportFunc;
use anyhow::{bail, Context, Result};
use wasmparser::{Import, Parser, Payload, RecGroup, TypeRef};

pub fn parse_import_funcs(wasm_binary: &[u8]) -> Result<Vec<ImportFunc>> {
    struct UntypedImportFunc<'a> {
        /// The module being imported from.
        pub module: &'a str,
        /// The name of the imported item.
        pub name: &'a str,
        /// The type index of the imported item.
        pub type_ref: u32,
    }
    impl<'a> TryFrom<Import<'a>> for UntypedImportFunc<'a> {
        type Error = anyhow::Error;

        fn try_from(import: Import<'a>) -> Result<Self> {
            if let TypeRef::Func(index) = import.ty {
                Ok(UntypedImportFunc {
                    module: import.module,
                    name: import.name,
                    type_ref: index,
                })
            } else {
                bail!("Import type is not a function");
            }
        }
    }

    let mut import_funcs: Vec<UntypedImportFunc> = Vec::new();
    let mut types: HashMap<usize, Rc<RecGroup>> = HashMap::new();

    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm_binary) {
        match payload? {
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import.context("Failed to parse import entry")?;
                    if matches!(import.ty, TypeRef::Func(..)) {
                        import_funcs.push(import.try_into().unwrap());
                    } else {
                        bail!("ImportSection other than Func is not supported");
                    }
                }
            }
            Payload::TypeSection(reader) => {
                for (offset, record_group) in reader.into_iter().enumerate() {
                    types.insert(
                        offset,
                        Rc::new(record_group.context("Failed to parse type")?),
                    );
                }
            }
            _ => {}
        }
    }

    let import_funcs: Vec<ImportFunc> = import_funcs
        .into_iter()
        .map(|func| ImportFunc {
            module: func.module,
            name: func.name,
            ty: types.get(&(func.type_ref as usize)).unwrap().clone(),
        })
        .collect();

    // all imported functions' type should be C ABI type.
    for func in import_funcs.iter() {
        if !func.is_c_abi() {
            bail!("Imported function type is not C ABI type");
        }
    }
    Ok(import_funcs)
}

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
        let wasi_args = desc_wasi_abi!(wasi_args(a, bb: i64, cc_c[8]));
        assert_eq!(wasi_args.name, "wasi_args");
        assert_eq!(wasi_args.args.len(), 3);
        assert_eq!(wasi_args.args[0].name, "a");
        assert_eq!(wasi_args.args[0].size, size_of::<wasi::DefaultAbiArgType>());
        assert_eq!(wasi_args.args[1].name, "bb");
        assert_eq!(wasi_args.args[1].size, size_of::<i64>());
        assert_eq!(wasi_args.args[2].name, "cc_c");
        assert_eq!(wasi_args.args[2].size, 8);
    }

    #[test]
    fn parse_import_funcs_test() {
        use wasmparser::ValType;

        let wasm_binary = wat::parse_str(
r#"(module $rust-672937185f5392fa.wasm
    (type (;0;) (func))
    (type (;1;) (func (param i32)))
    (type (;2;) (func (param i32 i32)))
    (type (;3;) (func (param i32 i32) (result i32)))
    (type (;4;) (func (param i32) (result i32)))
    (type (;5;) (func (param i32 i32 i32) (result i32)))
    (type (;6;) (func (param i32 i32 i32)))
    (type (;7;) (func (param i32 i64 i32) (result i32)))
    (type (;8;) (func (param i32 i32 i32 i32) (result i32)))
    (type (;9;) (func (result i32)))
    (type (;10;) (func (param i32) (result i64)))
    (type (;11;) (func (param i32 i32 i32 i32)))
    (type (;12;) (func (param i32 i64 i32)))
    (type (;13;) (func (param i64 i64 i32) (result i64)))
    (type (;14;) (func (param i32 i64 i64)))
    (type (;15;) (func (param i32 i32 i32 i32 i32)))
    (type (;16;) (func (param i32 i32 i32 i32 i32) (result i32)))
    (type (;17;) (func (param i32 i32 i64 i32)))
    (type (;18;) (func (param i32 i32 i64)))
    (type (;19;) (func (param i32 i32 i32 i32 i32 i32 i32)))
    (type (;20;) (func (param i32 i32 i32 i32 i32 i32) (result i32)))
    (type (;21;) (func (param i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32) (result i32)))
    (type (;22;) (func (param i32 i64 i32 i32 i32 i32 i32 i32) (result i32)))
    (type (;23;) (func (param i32 i32 i32 i32 i32 i32 i32) (result i32)))
    (type (;24;) (func (param i64 i32 i32) (result i32)))
    (type (;25;) (func (param i32 i64 i64 i64 i64)))
    (import "wasi_snapshot_preview1" "clock_time_get" (func $_ZN4wasi13lib_generated22wasi_snapshot_preview114clock_time_get17hf55eee9df40ce7deE (type 7)))
    (import "wasi_snapshot_preview1" "fd_write" (func $_ZN4wasi13lib_generated22wasi_snapshot_preview18fd_write17h12b230225e789f1eE (type 8)))
    (import "wasi_snapshot_preview1" "environ_get" (func $__imported_wasi_snapshot_preview1_environ_get (type 3)))
    (import "wasi_snapshot_preview1" "environ_sizes_get" (func $__imported_wasi_snapshot_preview1_environ_sizes_get (type 3)))
    (import "wasi_snapshot_preview1" "proc_exit" (func $__imported_wasi_snapshot_preview1_proc_exit (type 1)))
)"#).unwrap();
        let import_funcs = parse_import_funcs(&wasm_binary).unwrap();
        assert_eq!(import_funcs.len(), 5);
        assert_eq!(import_funcs[0].module, "wasi_snapshot_preview1");
        assert_eq!(import_funcs[0].name, "clock_time_get");
        assert!(import_funcs[0].is_c_abi());
        assert_eq!(import_funcs[0].unwrap_func().params().len(), 3);
        assert_eq!(import_funcs[0].unwrap_func().params()[1], ValType::I64);

        assert_eq!(import_funcs[4].name, "proc_exit");
        assert_eq!(import_funcs[4].unwrap_func().params().len(), 1);
        assert_eq!(import_funcs[4].unwrap_func().params()[0], ValType::I32);
    }
}
