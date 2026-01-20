/// This is a teeny Rust script which evaluates no-seatbelts on a given project.
/// The project has to be `ring_buffer_smoketest` for now.
use std::fs::{self};

use clap::Parser;

use crate::docker::{docker_compile, ensure_docker_image};
use crate::metrics::binary::analyze_elf;
use crate::metrics::size::get_size;
use crate::noseatbelts::{apply_suggestions, run_no_seatbelts};
use crate::{cli::EvalArgs, project::copy_dir_recursive};

mod cli;
mod docker;
mod metrics;
mod noseatbelts;
mod project;

use chrono::Utc;

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
    println!("exists? {}", baseline_path.exists());
    let baseline_size = get_size(baseline_path);
    let fixed_size = get_size(fixed_path);

    // 7. Save fixed project
    let fixed_out = project_dir.with_file_name(format!(
        "{}-no-seatbelts-fixed",
        project_dir.file_name().unwrap().to_str().unwrap()
    ));
    copy_dir_recursive(fixed_path, &fixed_out).expect("failed to save fixed project");

    // 8. Report + exit
    println!("Fixed project saved to {}", fixed_out.display());
    println!("Original size: {} bytes", baseline_size);
    println!("New size: {} bytes", fixed_size);
    println!(
        "This shrunk the binary by {} bytes",
        baseline_size as i64 - fixed_size as i64
    );

    assert!(fixed_size <= baseline_size, "fixed binary is not smaller!");

    // 9. Analyze ELF binaries.
    println!("BASELINE: ");
    let baseline_summary = analyze_elf(&repo_root, baseline_path);
    println!("{}", baseline_summary);
    println!("FIXED: ");
    let fixed_summary = analyze_elf(&repo_root, fixed_path);
    println!("{}", fixed_summary);

    let datetime_id = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let final_folder = project_dir
        .with_file_name("eval-runs")
        .join(datetime_id.to_string());
    fs::create_dir_all(&final_folder).expect("failed to create eval-runs folder");

    let final_baseline = final_folder.join("baseline");
    copy_dir_recursive(baseline_path, &final_baseline).expect("failed to save baseline project");

    let final_fixed = final_folder.join("fixed");
    copy_dir_recursive(fixed_path, &final_fixed).expect("failed to save fixed project");

    // build the report.
    let report_path = final_folder.join("panic-report.json");
    metrics::report::write_panic_report(
        &report_path,
        "ring-buffer-smoketest",
        &baseline_summary,
        &fixed_summary,
        baseline_size as usize,
        fixed_size as usize,
    )
    .expect("failed to write panic report");
}
