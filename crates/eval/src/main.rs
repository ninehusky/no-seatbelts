/// This is a teeny Rust script which evaluates no-seatbelts on a given project.
/// The project has to be `ring_buffer_smoketest` for now.
use std::{
    collections::{HashMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Output,
};

use clap::Parser;
use rustfix::{CodeFix, Filter, Suggestion};
use std::process::Command;
use tempfile::TempDir;

const TARGET: &str = "i686-unknown-linux-gnu";

#[derive(Debug, Parser)]
#[command(name = "eval")]
#[command(about = "Evaluate no-seatbelts on a given project", long_about = None)]
pub struct EvalArgs {
    pub src_path: PathBuf,
}

fn ensure_docker_image() {
    let in_ci = std::env::var("CI").is_ok();
    let force = std::env::var("EVAL_FORCE_DOCKER_BUILD").is_ok();

    if in_ci || force || !docker_image_exists("no-seatbelts-eval-env") {
        docker_build().expect("Failed to build docker image.");
    }
}

fn docker_image_exists(name: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn add_empty_workspace(cargo_toml: impl AsRef<Path>) {
    let mut f = OpenOptions::new()
        .append(true)
        .open(cargo_toml)
        .expect("open Cargo.toml");
    writeln!(f, "\n[workspace]").expect("write [workspace]");
}

fn docker_compile(repo_root: &Path, project_dir: &Path) -> Result<(), ()> {
    let rel = project_dir
        .strip_prefix(repo_root)
        .expect("project_dir not under repo_root");

    run_in_docker(
        repo_root,
        &[
            "cargo",
            "build",
            "--manifest-path",
            &format!("/work/{}/Cargo.toml", rel.display()),
            "--release",
            "--target",
            "i686-unknown-linux-gnu",
        ],
    );
    Ok(())
}

fn run_in_docker(repo_root: &Path, args: &[&str]) -> Output {
    let output = Command::new("docker")
        .args([
            "run",
            "--rm",
            "-e",
            "RUSTFLAGS=-C link-arg=-nostdlib",
            "-e",
            "CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=i686-linux-gnu-gcc",
            "-e",
            "AR=i686-linux-gnu-ar",
            "-v",
            &format!("{}:/work", repo_root.display()),
            "no-seatbelts-eval-env",
        ])
        .args(args)
        .output()
        .expect("failed to run docker command");
    if !output.status.success() {
        eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("docker command failed");
    }

    output
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_entry = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_entry)?;
        } else {
            fs::copy(entry.path(), dst_entry)?;
        }
    }
    Ok(())
}

fn run_no_seatbelts(repo_root: &Path, entry: &Path) -> Vec<Suggestion> {
    let output = Command::new("cargo")
        .current_dir(repo_root)
        .args([
            "run",
            "-p",
            "no-seatbelts",
            "--bin",
            "no-seatbelts",
            "--",
            entry.to_str().unwrap(),
            "--error-format=json",
            "--no-std",
        ])
        .output()
        .expect("failed to run no-seatbelts");

    if !output.status.success() {
        panic!("no-seatbelts failed to run");
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    let json_only: String = stderr
        .lines()
        .filter(|line| line.trim_start().starts_with('{'))
        .collect::<Vec<_>>()
        .join("\n");

    rustfix::get_suggestions_from_json(
        &json_only,
        &HashSet::<String>::default(),
        Filter::Everything,
    )
    .expect("failed to parse suggestions")
}

fn apply_suggestions(suggestions: &Vec<Suggestion>) {
    let mut fixes = HashMap::new();
    for suggestion in suggestions {
        let file_name = &suggestion.snippets[0].file_name;
        fixes
            .entry(file_name)
            .or_insert_with(Vec::new)
            .push(suggestion);
    }

    for (source_file, suggestions) in fixes {
        let source = fs::read_to_string(source_file).expect("Couldn't read source file");
        let mut fix = CodeFix::new(&source);
        for suggestion in suggestions.iter() {
            if let Err(e) = fix.apply(suggestion) {
                panic!("Failed to apply suggestion to {:?}: {}", source_file, e);
            }
        }
        let fixes = fix.finish().expect("Failed to finish applying fixes");
        fs::write(source_file, fixes).expect("Couldn't write fixed source file");
    }

    println!("applied {} fixes", suggestions.len());
}

fn get_size(crate_root: &Path) -> u64 {
    let target_dir = crate_root
        .join("target")
        .join(TARGET)
        .join("release")
        .join("deps");
    let mut total_size = 0;
    for entry in fs::read_dir(target_dir).expect("failed to read target/release/deps") {
        let entry = entry.expect("failed to read entry");
        if !entry
            .file_name()
            .to_str()
            .unwrap()
            .contains("ring_buffer_smoketest")
        {
            continue;
        }
        // make sure no file extension
        if entry.path().extension().is_some() {
            continue;
        }
        let metadata = entry.metadata().expect("failed to get metadata");
        total_size += metadata.len();
    }
    if total_size == 0 {
        panic!(
            "failed to find binary in target/release/deps. Are you calling get_size on ring_buffer_smoketest?"
        );
    }
    total_size
}

fn docker_build() -> Result<(), String> {
    let status = Command::new("docker")
        .args([
            "build",
            "-f",
            "crates/eval/docker/Dockerfile",
            "-t",
            "no-seatbelts-eval-env",
            ".",
        ])
        .status()
        .map_err(|e| format!("Failed to run docker build: {}", e))?;

    if !status.success() {
        return Err("Failed to build docker image".to_string());
    }

    Ok(())
}

fn main() {
    let repo_root = std::env::current_dir().expect("failed to get current dir");
    let args = EvalArgs::parse();

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
    let baseline_dir = TempDir::new_in(&repo_root).expect("failed to create tempdir");
    let baseline_tmp = baseline_dir.path();

    let fixed_dir = TempDir::new_in(&repo_root).expect("failed to create tempdir");
    let fixed_tmp = fixed_dir.path();

    copy_dir_recursive(project_dir, baseline_tmp).expect("failed to copy baseline project");
    add_empty_workspace(baseline_tmp.join("Cargo.toml"));
    copy_dir_recursive(project_dir, fixed_tmp).expect("failed to copy fixed project");
    add_empty_workspace(fixed_tmp.join("Cargo.toml"));

    // 3. In host, run no-seatbelts on fixed project
    let suggestions = run_no_seatbelts(&repo_root, fixed_tmp.join(relative_path).as_path());
    apply_suggestions(&suggestions);

    // 4. Build Docker image, if needed
    ensure_docker_image();

    // (Entering Docker now)
    // 5. Compile both projects inside Docker
    docker_compile(&repo_root, baseline_tmp).unwrap();
    docker_compile(&repo_root, fixed_tmp).unwrap();

    // 6. Measure ELF sizes (host-side)
    let baseline_size = get_size(baseline_tmp);
    let fixed_size = get_size(fixed_tmp);

    // 7. Save fixed project
    let fixed_out = project_dir.with_file_name(format!(
        "{}-no-seatbelts-fixed",
        project_dir.file_name().unwrap().to_str().unwrap()
    ));
    copy_dir_recursive(fixed_tmp, &fixed_out).expect("failed to save fixed project");

    // 8. Report + exit
    println!("Fixed project saved to {}", fixed_out.display());
    println!("Original size: {} bytes", baseline_size);
    println!("New size: {} bytes", fixed_size);
    println!(
        "This shrunk the binary by {} bytes",
        baseline_size as i64 - fixed_size as i64
    );

    if fixed_size >= baseline_size {
        std::process::exit(1);
    }
}
