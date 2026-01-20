/// Defines the core structs used to report panic-related metrics.
/// It's kind of a mess right now.
use std::{fmt::Display, path::Path};

use serde::Serialize;

use crate::metrics::binary::ElfAnalysis;

#[derive(Serialize)]
pub struct PanicReport {
    pub binary: String,
    pub baseline: BinaryPanicStats,
    pub fixed: BinaryPanicStats,
    pub diff: GlobalPanicDiff,
    pub functions: Vec<FunctionPanicDiff>,
}

#[derive(Serialize)]
pub struct FunctionPanicStats {
    pub size_bytes: usize,
    pub panic_calls: usize,
}

#[derive(Serialize)]
pub struct FunctionPanicDiff {
    pub name: String,
    pub baseline: FunctionPanicStats,
    pub fixed: FunctionPanicStats,
}

#[derive(Serialize)]
pub struct GlobalPanicDiff {
    pub size_delta: isize,
    pub removed_panic_calls: usize,
    pub removed_panic_functions: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct BinaryPanicStats {
    /// Counts
    pub num_functions: usize,
    pub num_panic_functions: usize,

    /// Sizes
    pub total_bytes: usize,

    /// Upper bound of potential savings
    pub removable_panic_function_bytes: usize,
    pub total_panic_calls: usize,
}

impl Display for BinaryPanicStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Panic Debloat Summary")?;
        writeln!(f, "---------------------")?;
        writeln!(f, "Total functions: {}", self.num_functions)?;
        writeln!(f, "Panic functions: {}", self.num_panic_functions)?;
        writeln!(f, "Total size (bytes): {}", self.total_bytes)?;
        writeln!(f, "Total panic calls: {}", self.total_panic_calls)?;
        writeln!(
            f,
            "Body of panic roots make up this many bytes: {}",
            self.removable_panic_function_bytes
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize)]
/// A function, brought in by the compiler, whose sole purpose is to
/// emit a panic.
pub struct PanicRootInfo {
    /// The name of the root function.
    name: String,
    /// Its size.
    pub size_bytes: usize,
}

impl PanicRootInfo {
    pub fn new(name: String, size_bytes: usize) -> Self {
        assert!(
            name.starts_with("core"),
            "Panic root functions are part of `core`."
        );
        Self { name, size_bytes }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct PanicCallSiteInfo {
    /// Name of the function containing the call
    pub caller: String,

    /// The panic function being called.
    pub callee: String,

    /// The size of the call instruction.
    pub call_size_bytes: usize,
}

/// Write a panic report to a JSON file.
/// This is the only entry point for report generation.
pub fn write_panic_report(
    out_path: &Path,
    binary_name: &str,
    baseline: &ElfAnalysis,
    fixed: &ElfAnalysis,
    baseline_size: usize,
    fixed_size: usize,
) -> Result<(), ()> {
    let report = build_panic_report(binary_name, baseline, fixed, baseline_size, fixed_size);

    let json =
        serde_json::to_string_pretty(&report).expect("failed to serialize panic report to JSON");
    std::fs::write(out_path, json).expect("failed to write panic report to file");

    Ok(())
}

fn build_panic_report(
    binary_name: &str,
    baseline: &ElfAnalysis,
    fixed: &ElfAnalysis,
    baseline_size: usize,
    fixed_size: usize,
) -> PanicReport {
    let diff = GlobalPanicDiff {
        size_delta: fixed_size as isize - baseline_size as isize,
        removed_panic_calls: count_panics(baseline) - count_panics(fixed),
        removed_panic_functions: baseline.summary.num_panic_functions
            - fixed.summary.num_panic_functions,
    };

    PanicReport {
        binary: binary_name.to_string(),
        baseline: baseline.summary.clone(),
        fixed: fixed.summary.clone(),
        diff,
        functions: compute_function_diffs(baseline, fixed),
    }
}

fn count_panics(analysis: &ElfAnalysis) -> usize {
    analysis.functions.values().map(|f| f.num_panic_calls).sum()
}

fn compute_function_diffs(baseline: &ElfAnalysis, fixed: &ElfAnalysis) -> Vec<FunctionPanicDiff> {
    let mut diffs = Vec::new();

    for (fn_name, base_fn) in &baseline.functions {
        if let Some(fixed_fn) = fixed.functions.get(fn_name) {
            let func_diff = FunctionPanicDiff {
                name: fn_name.clone(),
                baseline: FunctionPanicStats {
                    size_bytes: base_fn.total_bytes,
                    panic_calls: base_fn.num_panic_calls,
                },
                fixed: FunctionPanicStats {
                    size_bytes: fixed_fn.total_bytes,
                    panic_calls: fixed_fn.num_panic_calls,
                },
            };
            diffs.push(func_diff);
        }
    }

    diffs
}
