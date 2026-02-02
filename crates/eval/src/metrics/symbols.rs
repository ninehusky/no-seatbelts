use std::path::Path;

use rustc_demangle::demangle;

use crate::metrics::binary::analyze_elf;

/// This module defines the structures and functions for analyzing
/// assembly code to identify panic-related functions and calls.
///
/// Broadly, what this module does is check the "elf invariant":
/// for some harness, is it the case that it actually brings in the
/// functions used in the harness?

// Contains the names of Tock shim functions that must be present.
// Sorted by driver:
// - alarm.rs
// - button.rs
// - console.rs
// - led.rs
// - spi_peripheral.rs
// - stream.rs
// These aren't _technically_ shims, but their presence
// means the shims get called.
mod tock_shims {
    pub const ALARM_SHIMS: &[&str] = &[
        "KEEP_ALARM_NEW",
        // "KEEP_ALARM_EARLIEST_ALARM",
        // "KEEP_ALARM_PROCESS_REARM_OR_CALLBACK",
        // "KEEP_ALARM_REARM_U32_LEFT_JUSTIFIED_EXPIRATION",
        "KEEP_ALARM_COMMAND",
        // "KEEP_ALARM_ALLOCATE_GRANT",
        "KEEP_ALARM_ALARM",
    ];

    pub const BUTTON_SHIMS: &[&str] = &[
        "KEEP_BUTTON_NEW",
        "KEEP_BUTTON_GET_BUTTON_STATE",
        "KEEP_BUTTON_COMMAND",
        "KEEP_BUTTON_ALLOCATE_GRANT",
        "KEEP_BUTTON_FIRED",
    ];
}

fn is_shim(shim_name: &str, fn_name: &str) -> bool {
    fn_name.split("::").any(|part| part == shim_name)
}

pub fn check_elf_invariant(repo_root: &Path, elf_path: &Path) -> bool {
    println!("repo root: {}", repo_root.display());
    println!("elf path: {}", elf_path.display());

    let analysis = analyze_elf(repo_root, elf_path);

    for func in analysis.functions.keys() {
        println!("func: {}", demangle(func));
    }

    let all_shims = &[tock_shims::ALARM_SHIMS, tock_shims::BUTTON_SHIMS].concat();

    for shim in all_shims {
        if !analysis.functions.keys().any(|f| is_shim(shim, f.as_str())) {
            println!("Missing required shim: {}", shim);
            return false;
        }
    }

    println!("{}", analysis.summary);
    for (func, info) in analysis.functions.iter() {
        if info.is_panic_root {
            let demangled = demangle(func).to_string();
            println!("Panic root function: {}", demangled);
        }
    }

    println!("\n\n");

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
