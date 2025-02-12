use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser as ClapParser;
use wasi_guard::parse_import_funcs;

#[derive(ClapParser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    wasm_path: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let wasm_binary = std::fs::read(args.wasm_path).context("Failed to read WASM file")?;
    let funcs = parse_import_funcs(&wasm_binary).context("Error parsing WASM")?;
    for func in funcs {
        println!("{:?}", func);
    }
    Ok(())
}
