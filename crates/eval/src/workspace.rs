/// This module is purely for managing filesystem paths and related utilities.
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::transforms::{
    EditSummary,
    no_seatbelts::{apply_suggestions, run_no_seatbelts},
};
use crate::{docker::CompileConfig, transforms::EditMode};

/// This struct represents the temporary projects created for baseline and changed
/// versions. You typically want to build one of these through `prepare_benchmark_run`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PreparedProjects {
    pub baseline_path: PathBuf,
    pub edited_path: PathBuf,
    pub edit_summary: EditSummary,
}

/// Finds the root of the eval crate, which is expected to be the current working directory.
pub fn find_eval_root() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let cargo_toml = cwd.join("Cargo.toml");

    if !cargo_toml.exists() {
        anyhow::bail!("eval must be run from crates/eval (no Cargo.toml found)");
    }

    let contents = std::fs::read_to_string(&cargo_toml)?;
    if contents.contains("name = \"eval\"") {
        Ok(cwd)
    } else {
        anyhow::bail!(
            "eval must be run from crates/eval (found Cargo.toml, but not the eval crate)"
        );
    }
}

/// Finds the directory in which artifacts for the eval crate will be placed.
/// This is expected to be `target/eval` under the eval crate root.
pub fn find_build_dir() -> anyhow::Result<PathBuf> {
    let eval_root = find_eval_root()?;
    Ok(eval_root.join("build").join("latest"))
}

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

// Clones the project into two temporary directories, one for the baseline,
// and one for the new version.
pub fn prepare_benchmark_run_auto(
    benchmark_name: &str,
    benchmark_root: &Path,
    edit_mode: EditMode,
) -> anyhow::Result<PreparedProjects> {
    // 1. Set up temp directories for the two versions of the project.
    let build_dir = find_build_dir()?.join("benchmarks").join(benchmark_name);

    std::fs::remove_dir_all(&build_dir).ok();
    std::fs::create_dir_all(&build_dir)?;

    let baseline_path = build_dir.join("baseline");
    let edited_path = build_dir.join("edited");

    // 2. Clone the benchmark project into both temp directories.
    copy_dir_recursive(benchmark_root, &baseline_path)?;
    copy_dir_recursive(benchmark_root, &edited_path)?;

    // 3. Add empty workspaces to both Cargo.toml files to avoid dependency resolution issues.
    add_empty_workspace(baseline_path.join("Cargo.toml"));
    add_empty_workspace(edited_path.join("Cargo.toml"));

    let mut edit_summary = EditSummary::default();
    // 4. Apply transformations to the edited version, if needed.
    match edit_mode {
        EditMode::None => {}
        EditMode::NoSeatbelts => {
            let edits = run_no_seatbelts(&edited_path, &edited_path.join("src").join("lib.rs"))?;
            apply_suggestions(&edits);
            edit_summary.edit_mode = edit_mode;
            edit_summary.suggestions = edits.iter().map(|s| format!("{:?}", s)).collect();
        }
    };

    Ok(PreparedProjects {
        baseline_path,
        edited_path,
        edit_summary,
    })
}

pub fn expected_elf_path(project_dir: &Path, cfg: &CompileConfig) -> Option<PathBuf> {
    let bin = cfg.bin.as_ref()?;

    let profile = if cfg.release { "release" } else { "debug" };

    Some(
        project_dir
            .join("target")
            .join(cfg.target.to_rust_target())
            .join(profile)
            .join(bin),
    )
}
