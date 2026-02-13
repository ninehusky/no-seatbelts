use anyhow::Result;
use std::path::PathBuf;

use crate::{
    analysis::{
        functions,
        size::{self},
    },
    workspace::{find_build_dir, find_eval_root},
};

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PreparedProjects {
    baseline_path: PathBuf,
    fixed_path: PathBuf,
    project_dir: PathBuf,
    crate_relative_path: PathBuf,
}

pub fn run() -> Result<()> {
    let benchmark_name = "tock_harness";
    let target = crate::docker::TargetArch::Thumbv7emNoneEabi;

    // hard code the path to ./tock_harness_original and ./tock_harness_no_panic for now.
    let baseline_elf = find_eval_root()?
        .join("benchmarks")
        .join("tock_harness_original");
    let edited_elf = find_eval_root()?
        .join("benchmarks")
        .join("tock_harness_no_panic");

    // 1. EXPERIMENT 1: Compute overall size deltas.
    let section_size_summary = size::get_section_size_summary(&target, &baseline_elf, &edited_elf)?;

    // 2. EXPERIMENT 2: Compute function-level size deltas.
    let function_summary = functions::get_function_summary(&target, &baseline_elf, &edited_elf)?;

    // Also, as a helpful utility, find all the panic calls in the edited version.
    let panics = functions::find_panics(function_summary.edited.functions.clone());

    // Once done with both experiments, clone results to a final folder for persistence and reporting.
    let final_folder = find_build_dir()?.join("benchmarks").join(benchmark_name);
    // Add the JSON for the section size deltas.
    std::fs::create_dir_all(&final_folder)?;
    std::fs::write(
        final_folder.join("section-size-summary.json"),
        serde_json::to_string_pretty(&section_size_summary)?,
    )?;

    // Add the JSON for the function summary.
    std::fs::write(
        final_folder.join("function-summary.json"),
        serde_json::to_string_pretty(&function_summary)?,
    )?;

    std::fs::write(
        final_folder.join("remaining_panics.json"),
        serde_json::to_string_pretty(&panics)?,
    )?;

    println!("Wrote results to {}", final_folder.display());
    Ok(())
}
