use std::{
    path::Path,
    process::{Command, Output},
};

use std::path::PathBuf;

use anyhow::Context;

/// The way that the code should be compiled.
pub struct CompileConfig {
    pub target: TargetArch,
    pub bin: Option<String>,
    pub release: bool,
    pub exclude_std: bool,
}

/// The architectures you can compile to.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum TargetArch {
    I686UnknownLinuxGnu,
    Thumbv7emNoneEabi,
    X86_64UnknownLinuxGnu,
}

impl TargetArch {
    pub fn to_rust_target(&self) -> &'static str {
        match self {
            TargetArch::X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu",
            TargetArch::I686UnknownLinuxGnu => "i686-unknown-linux-gnu",
            TargetArch::Thumbv7emNoneEabi => "thumbv7em-none-eabi",
        }
    }

    pub fn is_call(&self, instr: &str) -> bool {
        // Get rid of the leading instruction address: just look at the mnemonic.
        fn mnemonic(asm: &str) -> &str {
            asm.split_whitespace()
                .find(|tok| {
                    // skip pure hex prefix bytes like "65", "f3", etc.
                    !tok.chars().all(|c| c.is_ascii_hexdigit())
                })
                .unwrap_or("")
        }

        let mnemonic = mnemonic(instr);

        match self {
            TargetArch::X86_64UnknownLinuxGnu | TargetArch::I686UnknownLinuxGnu => {
                mnemonic.starts_with("call")
            }
            TargetArch::Thumbv7emNoneEabi => mnemonic.starts_with("bl"),
        }
    }
}

pub fn ensure_docker_image() {
    let in_ci = std::env::var("CI").is_ok();

    // In CI, assume the image is already built by a CI step.
    // Only build locally when the image doesn't exist.
    if !in_ci && !docker_image_exists("no-seatbelts-eval-env") {
        docker_build().expect("Failed to build docker image.");
    }
}

pub fn docker_compile(
    mount_dir: &Path,
    project_dir: &Path,
    config: &CompileConfig,
) -> anyhow::Result<Option<PathBuf>> {
    // 1. Make sure the Docker image is built.
    ensure_docker_image();

    let rel = project_dir
        .strip_prefix(mount_dir)
        .context("project_dir must be under mount_dir")?;

    let cargo_toml_path = &format!("/work/{}/Cargo.toml", rel.display());

    let mut args = vec!["cargo", "build", "--manifest-path", cargo_toml_path];

    if config.release {
        args.push("--release");
    }

    if let Some(bin) = &config.bin {
        args.push("--bin");
        args.push(bin);
    }

    args.push("--target");
    args.push(config.target.to_rust_target());

    let env_args = if config.exclude_std {
        vec!["RUSTFLAGS=-C link-arg=-nostdlib"]
    } else {
        vec![]
    };

    run_in_docker(mount_dir, &args, &env_args)?;

    if let Some(elf) = crate::workspace::expected_elf_path(project_dir, config) {
        if !elf.exists() {
            anyhow::bail!(
                "cargo build succeeded, but expected ELF not found at {}",
                elf.display()
            );
        }
        return Ok(Some(elf));
    }
    Ok(None)
}

pub fn to_container_path(repo_root: &Path, host_path: &Path) -> String {
    let rel = host_path
        .strip_prefix(repo_root)
        .expect("path not under repo root");
    format!("/work/{}", rel.display())
}

pub fn run_in_docker(mount_dir: &Path, args: &[&str], env_args: &[&str]) -> anyhow::Result<Output> {
    ensure_docker_image();

    let base_env_args = [
        "CARGO_TARGET_I686_UNKNOWN_LINUX_GNU_LINKER=i686-linux-gnu-gcc",
        "AR=i686-linux-gnu-ar",
    ];

    let mount_dir_display = &format!("{}:/work", mount_dir.display());

    let mut prelude_args = vec!["run", "--rm", "--platform=linux/amd64"];

    for env_var in base_env_args.iter().chain(env_args) {
        prelude_args.push("-e");
        prelude_args.push(env_var);
    }

    prelude_args.push("-v");
    prelude_args.push(mount_dir_display);
    prelude_args.push("no-seatbelts-eval-env");

    let output = Command::new("docker")
        .args(&prelude_args)
        .args(args)
        .output()
        .expect("failed to run docker command");

    if !output.status.success() {
        anyhow::bail!(
            "Docker command failed with status {}. Stdout:\n{}\nStderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(output)
}

fn docker_image_exists(name: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn docker_build() -> Result<(), String> {
    let eval_root = crate::workspace::find_eval_root()
        .map_err(|e| format!("Failed to find eval root: {}", e))?;

    let status = Command::new("docker")
        .current_dir(&eval_root)
        .args([
            "build",
            "--platform=linux/amd64",
            "-f",
            "docker/Dockerfile",
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
