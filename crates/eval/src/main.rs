use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use clap::{Parser, builder::Str};
use rustfix::Filter;
use std::process::Command;
use tempfile::tempdir;

#[derive(Debug, Parser)]
#[command(name = "eval")]
#[command(about = "Evaluate no-seatbelts on a given project", long_about = None)]
pub struct EvalArgs {
    pub src_path: PathBuf,
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

    println!("copied to temp dir at {:?}", tmp_path);
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

    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "no-seatbelts",
            "--bin",
            "no-seatbelts",
            "--",
            tmp_entry.to_str().unwrap(),
            "--error-format=json",
            "--no-std",
        ])
        .output()
        .expect("failed to run no-seatbelts");

    if !output.status.success() {
        eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("cargo build failed.");
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let json_only: String = stderr
        .lines()
        .filter(|line| line.trim_start().starts_with('{'))
        .collect::<Vec<_>>()
        .join("\n");

    let mut error_codes: HashSet<String> = HashSet::default();
    let suggestions =
        rustfix::get_suggestions_from_json(&json_only, &error_codes, Filter::Everything)
            .expect("failed to parse suggestions");

    for suggestion in suggestions {
        println!("Suggestion:\n{:?}", suggestion);
    }
}
