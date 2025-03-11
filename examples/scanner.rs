use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use wasi_guard::{abi::forbidden_imports, parse_import_funcs, policy::policy, wasi::proc_exit};

policy! {
    default = allow;
    kill proc_exit;
}

#[derive(ClapParser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    wasm_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let wasm_binary = std::fs::read(args.wasm_path).context("Failed to read WASM file")?;
    let funcs = parse_import_funcs(&wasm_binary).context("Error parsing WASM")?;
    for func in forbidden_imports(&funcs, &MUST_BE_KILLED_WASIS) {
        println!("Fobidden: {}", func.name);
    }
    Ok(())
}
