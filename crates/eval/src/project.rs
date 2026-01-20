use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use tempfile::TempDir;

fn add_empty_workspace(cargo_toml: impl AsRef<Path>) {
    let mut f = OpenOptions::new()
        .append(true)
        .open(cargo_toml)
        .expect("open Cargo.toml");
    writeln!(f, "\n[workspace]").expect("write [workspace]");
}

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
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

pub fn prepare_temp_projects(repo_root: &Path, project_dir: &Path) -> (TempDir, TempDir) {
    let baseline_dir = TempDir::new_in(repo_root).expect("failed to create tempdir");
    let baseline_tmp = baseline_dir.path();

    let fixed_dir = TempDir::new_in(repo_root).expect("failed to create tempdir");
    let fixed_tmp = fixed_dir.path();

    copy_dir_recursive(project_dir, baseline_tmp).expect("failed to copy baseline project");
    add_empty_workspace(baseline_tmp.join("Cargo.toml"));
    copy_dir_recursive(project_dir, fixed_tmp).expect("failed to copy fixed project");
    add_empty_workspace(fixed_tmp.join("Cargo.toml"));

    println!(
        "created temp projects at {:?} and {:?}",
        baseline_tmp, fixed_tmp
    );

    println!("exists: {}", baseline_tmp.exists());
    println!("exists: {}", fixed_tmp.exists());

    (baseline_dir, fixed_dir)
}

pub fn find_elf(project_dir: &Path) -> std::path::PathBuf {
    let target_dir = project_dir
        .join("target")
        .join("i686-unknown-linux-gnu")
        .join("release");
    for entry in fs::read_dir(target_dir).expect("failed to read target/release/deps") {
        let entry = entry.expect("failed to read entry");
        if !entry
            .file_name()
            .to_str()
            .unwrap()
            .contains("ring-buffer-smoketest")
        {
            continue;
        }
        // make sure no file extension
        if entry.path().extension().is_some() {
            continue;
        }
        return entry.path();
    }
    panic!(
        "failed to find binary in target/release/deps. Are you calling find_elf on ring_buffer_smoketest?"
    );
}
