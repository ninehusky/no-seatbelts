use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};

use clap::Parser;
use rustfix::{CodeFix, Filter};
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

    let suggestions = rustfix::get_suggestions_from_json(
        &json_only,
        &HashSet::<String>::default(),
        Filter::Everything,
    )
    .expect("failed to parse suggestions");

    let mut fixes = HashMap::new();
    for suggestion in &suggestions {
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
                eprintln!("Failed to apply suggestion to {}: {}", source_file, e);
            }
        }
        let fixes = fix.finish().expect("Failed to finish applying fixes");
        fs::write(&source_file, fixes).expect("Couldn't write fixed source file");
    }

    println!("applied {} fixes", suggestions.len());

    // now, compile the fixed project.
    let output = Command::new("cargo")
        .args(["build"])
        .current_dir(tmp_path)
        .output()
        .expect("failed to run cargo build on fixed project");

    if !output.status.success() {
        eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        panic!("cargo build failed after applying fixes.");
    }

    println!("project compiled successfully after applying fixes.");
}
