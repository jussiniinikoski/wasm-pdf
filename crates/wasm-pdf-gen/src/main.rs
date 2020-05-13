//! # WASM-PDF CLI & WASI App
//!
//! This program can be compiled to a non-browser CLI application,
//! and also be used with wasmtime and other apps with WASI capability.

use std::env;
use wasm_pdf_gen::files::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    if args.len() < 3 {
        eprintln!("{} <input.json> <output.pdf>", program);
        return;
    }

    if !&args[1].ends_with(".json") {
        eprintln!("Input file should be a JSON (.json) file.");
        return;
    }

    if !&args[2].ends_with(".pdf") {
        eprintln!("Output file should be PDF (.pdf) file.");
        return;
    }

    if let Err(err) = process(&args[1], &args[2]) {
        eprintln!("{}", err)
    }
}
