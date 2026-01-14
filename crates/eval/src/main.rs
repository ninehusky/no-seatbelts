/// This is a teeny Rust script which evaluates no-seatbelts on a given project.
/// The project has to be `ring_buffer_smoketest` for now.
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use rustfix::{CodeFix, Filter, Suggestion};
use std::process::Command;
use tempfile::tempdir;

#[derive(Debug, Parser)]
#[command(name = "eval")]
#[command(about = "Evaluate no-seatbelts on a given project", long_about = None)]
pub struct EvalArgs {
    pub src_path: PathBuf,
}

fn compile_crate(project_dir: &Path) -> Result<PathBuf, ()> {
    // copy the project to a temp dir
    let mut command = Command::new("cargo");

    command.args(["build"]).current_dir(project_dir);
    command.env("RUSTFLAGS", "-C link-arg=-nostdlib");
    command.arg("--release");

    let output = command
        .output()
        .expect("failed to run cargo build on project");

    if !output.status.success() {
        eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        return Err(());
    }

    let target_dir = project_dir.join("target").join("release");
    Ok(target_dir)
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

fn run_no_seatbelts(entry: &Path) -> Vec<Suggestion> {
    let output = Command::new("cargo")
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
        let file_name = suggestion.snippets[0].file_name.clone();
        fixes
            .entry(file_name)
            .or_insert_with(Vec::new)
            .push(suggestion);
    }

    for (source_file, suggestions) in fixes {
        let source = fs::read_to_string(&source_file).expect("Couldn't read source file");
        let mut fix = CodeFix::new(&source);
        for suggestion in suggestions.iter() {
            if let Err(e) = fix.apply(suggestion) {
                panic!("Failed to apply suggestion to {}: {}", source_file, e);
            }
        }
        let fixes = fix.finish().expect("Failed to finish applying fixes");
        fs::write(&source_file, fixes).expect("Couldn't write fixed source file");
    }

    println!("applied {} fixes", suggestions.len());
}

fn get_size(crate_root: &Path) -> u64 {
    let target_dir = crate_root.join("target").join("release").join("deps");
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

fn main() {
    let args = EvalArgs::parse();

    let crate_root = &args.src_path;

    if !crate_root.is_file() {
        panic!("For now, we expect src_path to be src/lib.rs or src/main.rs");
    }

    let src_dir = crate_root.parent().expect("failed to get parent dir");
    let project_dir = src_dir.parent().expect("failed to get project dir");

    let tmp = tempdir().expect("failed to create tempdir");
    let tmp_path = tmp.path();

    copy_dir_recursive(project_dir, tmp_path).expect("failed to copy project");

    let relative_entry = crate_root
        .strip_prefix(project_dir)
        .expect("crate root should be under project root");

    let tmp_entry = tmp_path.join(relative_entry);

    if !tmp_entry.is_file() {
        panic!(
            "expected temp crate root file at {}, but it does not exist",
            tmp_entry.display()
        );
    }

    let suggestions = run_no_seatbelts(&tmp_entry);
    apply_suggestions(&suggestions);

    let clone_of_project_dir = tempdir().expect("failed to create tempdir");
    let clone_of_project_path = clone_of_project_dir.path();
    copy_dir_recursive(project_dir, clone_of_project_path).expect("failed to copy project");

    // now, compile the fixed project.
    compile_crate(clone_of_project_path).expect("failed to compile original project");
    compile_crate(tmp_path).expect("failed to compile fixed project");

    // report the sizes of everything in target/release/deps
    let og_size: u64 = get_size(clone_of_project_path);
    let new_size: u64 = get_size(tmp_path);

    // finally, save the fixed project to a new location
    let fixed_project_path = project_dir.with_file_name(format!(
        "{}-no-seatbelts-fixed",
        project_dir.file_name().unwrap().to_str().unwrap()
    ));

    copy_dir_recursive(tmp_path, &fixed_project_path)
        .expect("failed to copy fixed project to final location");

    println!("Fixed project saved to {}", fixed_project_path.display());

    println!("Original size: {} bytes", og_size);
    println!("New size: {} bytes", new_size);
    println!(
        "This shrunk the binary by {} bytes",
        og_size as i64 - new_size as i64
    );
}
