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
    pub const ALARM_SHIMS: &[&str] = &["KEEP_ALARM_NEW", "KEEP_ALARM_COMMAND", "KEEP_ALARM_ALARM"];

    pub const BUTTON_SHIMS: &[&str] = &[
        "KEEP_BUTTON_NEW",
        "KEEP_BUTTON_GET_BUTTON_STATE",
        "KEEP_BUTTON_COMMAND",
        "KEEP_BUTTON_ALLOCATE_GRANT",
        "KEEP_BUTTON_FIRED",
    ];

    pub const CONSOLE_SHIMS: &[&str] = &[
        "KEEP_CONSOLE_NEW",
        "KEEP_CONSOLE_COMMAND",
        "KEEP_CONSOLE_ALLOCATE_GRANT",
        "KEEP_CONSOLE_TRANSMITTED_BUFFER",
        "KEEP_CONSOLE_RECEIVED_BUFFER",
    ];

    pub const LED_SHIMS: &[&str] = &[
        "KEEP_LEDLOW_NEW",
        "KEEP_LEDLOW_INIT",
        "KEEP_LEDLOW_ON",
        "KEEP_LEDLOW_OFF",
        "KEEP_LEDLOW_TOGGLE",
        "KEEP_LEDLOW_READ",
        "KEEP_LEDHIGH_NEW",
        "KEEP_LEDHIGH_INIT",
        "KEEP_LEDHIGH_ON",
        "KEEP_LEDHIGH_OFF",
        "KEEP_LEDHIGH_TOGGLE",
        "KEEP_LEDHIGH_READ",
    ];

    pub const SPI_PERIPHERAL_SHIMS: &[&str] = &[
        "KEEP_SPI_PERIPHERAL_NEW",
        "KEEP_SPI_PERIPHERAL_CONFIG_BUFFERS",
        "KEEP_SPI_PERIPHERAL_READ_WRITE_DONE",
        "KEEP_SPI_PERIPHERAL_ALLOCATE_GRANT",
        "KEEP_SPI_PERIPHERAL_CHIP_SELECTED",
    ];

    pub const STREAM_SHIMS: &[&str] = &[
        "KEEP_STREAM_SRESULT_IS_DONE",
        "KEEP_STREAM_SRESULT_IS_ERR",
        "KEEP_STREAM_SRESULT_DONE",
        "KEEP_STREAM_SRESULT_NEEDED",
        "KEEP_STREAM_SRESULT_ERR",
        "KEEP_STREAM_ENCODE_U8",
        "KEEP_STREAM_ENCODE_U16",
        "KEEP_STREAM_ENCODE_U32",
        "KEEP_STREAM_ENCODE_BYTES",
        "KEEP_STREAM_ENCODE_BYTES_BE",
        "KEEP_STREAM_DECODE_U8",
        "KEEP_STREAM_DECODE_U16",
        "KEEP_STREAM_DECODE_U32",
        "KEEP_STREAM_DECODE_BYTES",
        "KEEP_STREAM_DECODE_BYTES_BE",
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

    let all_shims = &[
        tock_shims::ALARM_SHIMS,
        tock_shims::BUTTON_SHIMS,
        tock_shims::CONSOLE_SHIMS,
        tock_shims::LED_SHIMS,
        tock_shims::SPI_PERIPHERAL_SHIMS,
        tock_shims::STREAM_SHIMS,
    ]
    .concat();

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
