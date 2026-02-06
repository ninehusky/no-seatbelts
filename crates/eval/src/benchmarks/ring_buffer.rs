use anyhow::Result;
use std::path::PathBuf;

use crate::{
    analysis::{SectionSizeDeltas, SectionSizes, size::get_section_sizes},
    docker::{CompileConfig, docker_compile},
    workspace::{self, prepare_benchmark_run_auto},
};

#[derive(Debug, Clone)]
struct PreparedProjects {
    baseline_path: PathBuf,
    fixed_path: PathBuf,
    project_dir: PathBuf,
    crate_relative_path: PathBuf,
}

pub fn run() -> Result<()> {
    let eval_root = workspace::find_eval_root()?;
    let prepared = prepare_benchmark_run_auto(
        "ring_buffer",
        &eval_root.join("benchmarks").join("ring_buffer"),
        workspace::EditMode::NoSeatbelts,
    )?;

    let compile_cfg = CompileConfig {
        target: crate::docker::TargetArch::I686_UNKNOWN_LINUX_GNU,
        bin: Some("ring-buffer-smoketest".to_string()),
        release: true,
    };

    let baseline_elf = docker_compile(&eval_root, &prepared.baseline_path, &compile_cfg)?.unwrap();
    let edited_elf = docker_compile(&eval_root, &prepared.edited_path, &compile_cfg)?.unwrap();

    let baseline_sizes = get_section_sizes(&baseline_elf)?;
    let edited_sizes = get_section_sizes(&edited_elf)?;

    let mut delta: SectionSizeDeltas = Default::default();
    for (section, baseline_size) in baseline_sizes.iter() {
        let edited_size = edited_sizes.get(section).copied().unwrap();
        delta.insert(
            section.clone(),
            (edited_size as i64 - (*baseline_size as i64)) as i64,
        );
    }

    println!("diff:");
    for (k, v) in &delta {
        println!("{}: {}", k, v);
    }

    // Part 1: Prepare baseline + transformed source trees.
    {
        // 1. Remove previous `fixed` version, if any.
        // let crate_dir = args
        //     .src_path
        //     .parent()
        //     .and_then(|p| p.parent())
        //     .expect("failed to get crate dir");

        // let fixed_out = crate_dir.with_file_name(format!(
        //     "{}-no-seatbelts-fixed",
        //     crate_dir.file_name().unwrap().to_str().unwrap()
        // ));

        // if fixed_out.exists() {
        //     fs::remove_dir_all(&fixed_out).expect("failed to remove previous fixed project");
        //     println!("Removed previous fixed project at {}", fixed_out.display());
        //     println!("Hope you didn't need it!");
        // }

        // // 1. Identify project layout
        // let crate_root = &args.src_path;
        // if !crate_root.is_file() {
        //     panic!("expected src/lib.rs or src/main.rs");
        // }

        // let src_dir = crate_root.parent().expect("no src dir");
        // let project_dir = src_dir.parent().expect("no project dir");

        // let relative_path = crate_root
        //     .strip_prefix(project_dir)
        //     .expect("path not under project root");

        // // 2. Prepare temp dirs (in host FS)
        // let (baseline_dir, fixed_dir) = project::prepare_temp_projects(&repo_root, project_dir);
        // let baseline_path = baseline_dir.path();
        // let fixed_path = fixed_dir.path();

        // // 3. In host, run no-seatbelts on fixed project
        // let suggestions = run_no_seatbelts(&repo_root, fixed_path.join(relative_path).as_path());

        // // let suggestions = run_no_seatbelts(&repo_root, fixed_dir.join(relative_path).as_path());
        // apply_suggestions(&suggestions);
    }

    // Part 2: Compile the baseline and transformed projects in Docker
    {
        // // 4. Build Docker image, if needed
        // ensure_docker_image();

        // // (Entering Docker now)
        // // 5. Compile both projects inside Docker
        // docker_compile(&repo_root, baseline_path).unwrap();
        // docker_compile(&repo_root, fixed_path).unwrap();

        // // 6. Measure ELF sizes (host-side)

        // // 7. Save fixed project
        // let fixed_out = project_dir.with_file_name(format!(
        //     "{}-no-seatbelts-fixed",
        //     project_dir.file_name().unwrap().to_str().unwrap()
        // ));
        // copy_dir_recursive(fixed_path, &fixed_out).expect("failed to save fixed project");

        // // 8. Report + exit
        // println!("Fixed project saved to {}", fixed_out.display());
    }

    // Part 3: Analyze and report results
    {
        // // 9. Analyze ELF binaries.
        // let baseline_summary = analyze_elf(&repo_root, baseline_path);
        // let fixed_summary = analyze_elf(&repo_root, fixed_path);

        // let final_folder = project_dir.with_file_name("eval-runs").join("latest");
        // fs::create_dir_all(&final_folder).expect("failed to create eval-runs folder");

        // let final_baseline = final_folder.join("baseline");
        // copy_dir_recursive(baseline_path, &final_baseline).expect("failed to save baseline project");

        // let final_fixed = final_folder.join("fixed");
        // copy_dir_recursive(fixed_path, &final_fixed).expect("failed to save fixed project");

        // // 10. Build the report.
        // let report_path = final_folder.join("panic-report.json");
        // let asm_path = final_folder.join("asm-dumps");
        // fs::create_dir_all(&asm_path).expect("failed to create asm-dumps folder");

        // for fn_name in baseline_summary.functions.keys() {
        //     let sanitized_fn_name = sanitize_for_path(fn_name);
        //     let fn_path = asm_path.join(&sanitized_fn_name);
        //     fs::create_dir_all(&fn_path).expect("failed to create function asm folder");
        //     let baseline_asm_path = fn_path.join(format!("baseline-{}.asm", sanitized_fn_name));
        //     let fixed_asm_path = fn_path.join(format!("fixed-{}.asm", sanitized_fn_name));

        //     if let Some(fn_asm) = baseline_summary.functions.get(fn_name) {
        //         fs::write(&baseline_asm_path, &fn_asm.body)
        //             .expect("failed to write baseline function asm");
        //     }

        //     if let Some(fn_asm) = fixed_summary.functions.get(fn_name) {
        //         fs::write(&fixed_asm_path, &fn_asm.body).expect("failed to write fixed function asm");
        //     } else {
        //         fs::write(&fixed_asm_path, "// Function removed in fixed build")
        //             .expect("failed to write fixed function asm");
        //     }
        // }

        // metrics::report::write_panic_report(
        //     &report_path,
        //     "ring-buffer-smoketest",
        //     &baseline_summary,
        //     &fixed_summary,
        //     metrics::size::get_size_report(&repo_root, baseline_path),
        //     metrics::size::get_size_report(&repo_root, fixed_path),
        // )
        // .expect("failed to write panic report");
    }
    Ok(())
}
