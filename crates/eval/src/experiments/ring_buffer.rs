use anyhow::Result;
use std::path::PathBuf;

use crate::{
    analysis::{
        functions,
        size::{self},
    },
    docker::{CompileConfig, docker_compile},
    transforms::EditMode,
    workspace::{self, find_build_dir, prepare_benchmark_run_auto},
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
    let eval_root = workspace::find_eval_root()?;
    let target = crate::docker::TargetArch::I686UnknownLinuxGnu;
    let benchmark_name = "ring_buffer";
    let prepared = prepare_benchmark_run_auto(
        benchmark_name,
        &eval_root.join("benchmarks").join("ring_buffer"),
        EditMode::NoSeatbelts,
    )?;

    let compile_cfg = CompileConfig {
        target: target.clone(),
        bin: Some("ring-buffer-smoketest".to_string()),
        release: true,
        exclude_std: true,
    };

    // 1. EXPERIMENT 1: Compute overall size deltas.
    let baseline_elf = docker_compile(&eval_root, &prepared.baseline_path, &compile_cfg)?.unwrap();
    let edited_elf = docker_compile(&eval_root, &prepared.edited_path, &compile_cfg)?.unwrap();

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
