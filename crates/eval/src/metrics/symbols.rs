use std::path::Path;

use rustc_demangle::demangle;

use crate::{docker::run_in_docker, metrics::binary::analyze_elf};

/// This module defines the structures and functions for analyzing
/// assembly code to identify panic-related functions and calls.
///
/// Broadly, what this module does is check the "elf invariant":
/// for some harness, is it the case that it actually brings in the
/// functions used in the harness?

const BUTTON_REQUIRED_SHIMS: &[&str] = &[];

pub fn check_elf_invariant(repo_root: &Path, elf_path: &Path) -> bool {
    println!("repo root: {}", repo_root.display());
    println!("elf path: {}", elf_path.display());

    let analysis = analyze_elf(repo_root, elf_path);

    println!("{}", analysis.summary);

    for (func, info) in analysis.functions.iter() {
        let panic_calls = &info.panic_calls;
        if panic_calls.is_empty() {
            continue;
        }
        let demangled = demangle(func).to_string();
        // print the panic calls
        println!("func: {}", demangled);
        for pc in panic_calls {
            println!(
                "  panic call from {} to {}",
                demangle(pc.caller.as_str()),
                demangle(pc.callee.as_str())
            );
        }
    }

    true
}
