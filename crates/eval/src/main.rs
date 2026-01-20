/// This is a teeny Rust script which evaluates no-seatbelts on a given project.
/// The project has to be `ring_buffer_smoketest` for now.
use std::fs::{self};

use clap::Parser;

use crate::docker::{docker_compile, ensure_docker_image};
use crate::metrics::binary::analyze_elf;
use crate::noseatbelts::{apply_suggestions, run_no_seatbelts};
use crate::{cli::EvalArgs, project::copy_dir_recursive};

mod cli;
mod docker;
mod metrics;
mod noseatbelts;
mod project;

#[allow(dead_code)]
const TARGET: &str = "i686-unknown-linux-gnu";

fn main() {
    let repo_root = std::env::current_dir().expect("failed to get current dir");
    let args = EvalArgs::parse();

    // 1. Remove previous `fixed` version, if any.
    let crate_dir = args
        .src_path
        .parent()
        .and_then(|p| p.parent())
        .expect("failed to get crate dir");

    let fixed_out = crate_dir.with_file_name(format!(
        "{}-no-seatbelts-fixed",
        crate_dir.file_name().unwrap().to_str().unwrap()
    ));

    if fixed_out.exists() {
        fs::remove_dir_all(&fixed_out).expect("failed to remove previous fixed project");
        println!("Removed previous fixed project at {}", fixed_out.display());
        println!("Hope you didn't need it!");
    }

    // 1. Identify project layout
    let crate_root = &args.src_path;
    if !crate_root.is_file() {
        panic!("expected src/lib.rs or src/main.rs");
    }

    let src_dir = crate_root.parent().expect("no src dir");
    let project_dir = src_dir.parent().expect("no project dir");

    let relative_path = crate_root
        .strip_prefix(project_dir)
        .expect("path not under project root");

    // 2. Prepare temp dirs (in host FS)
    let (baseline_dir, fixed_dir) = project::prepare_temp_projects(&repo_root, project_dir);
    let baseline_path = baseline_dir.path();
    let fixed_path = fixed_dir.path();

    // 3. In host, run no-seatbelts on fixed project
    let suggestions = run_no_seatbelts(&repo_root, fixed_path.join(relative_path).as_path());

    // let suggestions = run_no_seatbelts(&repo_root, fixed_dir.join(relative_path).as_path());
    apply_suggestions(&suggestions);

    // 4. Build Docker image, if needed
    ensure_docker_image();

    // (Entering Docker now)
    // 5. Compile both projects inside Docker
    docker_compile(&repo_root, baseline_path).unwrap();
    docker_compile(&repo_root, fixed_path).unwrap();

    // 6. Measure ELF sizes (host-side)

    // 7. Save fixed project
    let fixed_out = project_dir.with_file_name(format!(
        "{}-no-seatbelts-fixed",
        project_dir.file_name().unwrap().to_str().unwrap()
    ));
    copy_dir_recursive(fixed_path, &fixed_out).expect("failed to save fixed project");

    // 8. Report + exit
    println!("Fixed project saved to {}", fixed_out.display());

    // 9. Analyze ELF binaries.
    let baseline_summary = analyze_elf(&repo_root, baseline_path);
    let fixed_summary = analyze_elf(&repo_root, fixed_path);

    let final_folder = project_dir.with_file_name("eval-runs").join("latest");
    fs::create_dir_all(&final_folder).expect("failed to create eval-runs folder");

    let final_baseline = final_folder.join("baseline");
    copy_dir_recursive(baseline_path, &final_baseline).expect("failed to save baseline project");

    let final_fixed = final_folder.join("fixed");
    copy_dir_recursive(fixed_path, &final_fixed).expect("failed to save fixed project");

    // 10. Build the report.
    let report_path = final_folder.join("panic-report.json");
    let asm_path = final_folder.join("asm-dumps");
    fs::create_dir_all(&asm_path).expect("failed to create asm-dumps folder");

    for fn_name in baseline_summary.functions.keys() {
        let fn_file_name = fn_name.replace("<", "_").replace(">", "_");
        let fn_path = asm_path.join(fn_name.clone());
        fs::create_dir_all(&fn_path).expect("failed to create function asm folder");
        let baseline_asm_path = fn_path.join(format!("baseline-{}.asm", fn_file_name));
        let fixed_asm_path = fn_path.join(format!("fixed-{}.asm", fn_file_name));

        if let Some(fn_asm) = baseline_summary.functions.get(fn_name) {
            fs::write(&baseline_asm_path, &fn_asm.body)
                .expect("failed to write baseline function asm");
        }

        if let Some(fn_asm) = fixed_summary.functions.get(fn_name) {
            fs::write(&fixed_asm_path, &fn_asm.body).expect("failed to write fixed function asm");
        } else {
            fs::write(&fixed_asm_path, "// Function removed in fixed build")
                .expect("failed to write fixed function asm");
        }
    }

    metrics::report::write_panic_report(
        &report_path,
        "ring-buffer-smoketest",
        &baseline_summary,
        &fixed_summary,
        metrics::size::get_size_report(baseline_path),
        metrics::size::get_size_report(fixed_path),
    )
    .expect("failed to write panic report");
}
