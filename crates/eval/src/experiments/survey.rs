use serde::{Deserialize, Serialize};

use crate::{
    docker::{CompileConfig, docker_compile},
    workspace::{find_build_dir, find_eval_root},
};

#[derive(Serialize, Deserialize)]
struct BinaryCrate {
    name: String,
    exec_name: String,
    version: String,
    repository: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct BinaryPanicReport {
    name: String,
    num_calls: u64,
    num_panics: u64,
    panic_proportion: f64,
}

#[derive(Serialize, Deserialize)]
struct SurveyReport {
    binary_reports: Vec<BinaryPanicReport>,
    total_calls: u64,
    total_panics: u64,
    panic_proportion: f64,
}

fn to_cloneable_url(repo: &str) -> Option<String> {
    if repo.starts_with("http") {
        Some(format!("{}.git", repo.trim_end_matches(".git")))
    } else {
        None
    }
}

pub fn run() -> anyhow::Result<()> {
    let survey_repo_dir = find_build_dir()?.join("benchmarks").join("survey_repos");
    // Get rid of any previously cloned repos, to start fresh.
    if survey_repo_dir.exists() {
        std::fs::remove_dir_all(&survey_repo_dir)?;
    }
    std::fs::create_dir_all(&survey_repo_dir)?;

    // 1. Read from find_eval_root()/data/interesting_crates.json.
    let crates = crate::workspace::find_eval_root()?.join("data/interesting_crates.json");
    let data = std::fs::read_to_string(crates)?;
    let interesting_crates: Vec<BinaryCrate> = serde_json::from_str(&data)?;

    let mut binary_reports = Vec::new();

    for krate in interesting_crates {
        let cfg = CompileConfig {
            target: crate::docker::TargetArch::I686UnknownLinuxGnu,
            // target: crate::docker::TargetArch::X86_64UnknownLinuxGnu,
            bin: Some(krate.exec_name.clone()),
            release: true,
        };

        // 2. Clone the repo if it exists.
        let Some(repo) = krate.repository else {
            println!(
                "Crate {} does not have a repository listed, skipping.",
                krate.name
            );
            continue;
        };
        let Some(repo) = to_cloneable_url(&repo) else {
            println!(
                "Crate {} has a repository URL that doesn't look cloneable ({}), skipping.",
                krate.name, repo
            );
            continue;
        };

        println!("Cloning {} from {}...", krate.name, repo);
        let output = std::process::Command::new("git")
            .current_dir(&survey_repo_dir)
            .args(["clone", &repo, "--depth", "1"])
            .output()?;

        if !output.status.success() {
            anyhow::bail!(
                "Failed to clone {} from {}. Status: {}. Stdout:\n{}\nStderr:\n{}",
                krate.name,
                repo,
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }

        // 3. Find the Cargo.toml.
        let cargo_toml_path = survey_repo_dir.join(&krate.name).join("Cargo.toml");
        // if "strip = true" is in the Cargo.toml, we need to remove it, since it prevents us from analyzing the binary.
        let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path)?;
        if cargo_toml_content.contains("strip = true") {
            println!(
                "Cargo.toml for {} contains 'strip = true', removing it for analysis.",
                krate.name
            );
            let new_content = cargo_toml_content.replace("strip = true", "");
            std::fs::write(&cargo_toml_path, new_content)?;
        }

        // 3. Try to build the repo with cargo build --release, and report success or failure.
        let cloned_dir = survey_repo_dir.join(&krate.name);
        docker_compile(&find_eval_root()?, &cloned_dir, &cfg)?;

        let elf_path = cloned_dir
            .join("target")
            .join(cfg.target.to_rust_target())
            .join("release")
            .join(&krate.exec_name);

        // 4. Count the number of panics in the binary.
        let section_size_summary =
            crate::analysis::size::get_section_size_summary(&cfg.target, &elf_path, &elf_path)?;

        let function_summary =
            crate::analysis::functions::get_function_summary(&cfg.target, &elf_path, &elf_path)?;

        let panics =
            crate::analysis::functions::find_panics(function_summary.edited.functions.clone());

        binary_reports.push(BinaryPanicReport {
            name: krate.name.clone(),
            num_calls: function_summary.edited.functions.len() as u64,
            num_panics: panics.len() as u64,
            panic_proportion: if function_summary.edited.functions.is_empty() {
                0.0
            } else {
                panics.len() as f64 / function_summary.edited.functions.len() as f64
            },
        });

        // Once done with both experiments, clone results to a final folder for persistence and reporting.
        let final_folder = find_build_dir()?
            .join("benchmarks")
            .join("survey_repos")
            .join("survey_results")
            .join(krate.name.clone());
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
    }

    // 5. Generate a report summarizing the results.
    let survey_report = SurveyReport {
        total_calls: binary_reports.iter().map(|r| r.num_calls).sum(),
        total_panics: binary_reports.iter().map(|r| r.num_panics).sum(),
        panic_proportion: if binary_reports.iter().map(|r| r.num_calls).sum::<u64>() == 0 {
            0.0
        } else {
            binary_reports.iter().map(|r| r.num_panics).sum::<u64>() as f64
                / binary_reports.iter().map(|r| r.num_calls).sum::<u64>() as f64
        },
        binary_reports,
    };

    std::fs::write(
        find_build_dir()?
            .join("benchmarks")
            .join("survey_repos")
            .join("survey_report.json"),
        serde_json::to_string_pretty(&survey_report)?,
    )?;

    Ok(())
}
