use std::{
    path::Path,
    process::{Command, Output},
};

pub fn ensure_docker_image() {
    let in_ci = std::env::var("CI").is_ok();
    let force = std::env::var("EVAL_FORCE_DOCKER_BUILD").is_ok();

    if in_ci || force || !docker_image_exists("no-seatbelts-eval-env") {
        docker_build().expect("Failed to build docker image.");
    }
}

pub fn docker_compile(repo_root: &Path, project_dir: &Path) -> Result<(), ()> {
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

pub fn run_in_docker(repo_root: &Path, args: &[&str]) -> Output {
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

fn docker_image_exists(name: &str) -> bool {
    Command::new("docker")
        .args(["image", "inspect", name])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
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
