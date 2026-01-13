use std::{path::PathBuf, process::Command};

use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "eval")]
#[command(about = "Evaluate no-seatbelts on a given project", long_about = None)]
pub struct EvalArgs {
    pub src_path: PathBuf,
}

fn main() {
    let args = EvalArgs::parse();
    println!("Running no-seatbelts on project at {:?}", args.src_path);

    use std::process::Command;

    let output = Command::new("cargo")
        .args([
            "run",
            "-p",
            "no-seatbelts",
            "--bin",
            "no-seatbelts",
            "--",
            args.src_path.to_str().unwrap(),
            "--error-format=json",
        ])
        .output()
        .expect("failed to run no-seatbelts");

    if !output.status.success() {
        eprintln!("cargo build failed");
        eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    eprintln!("suggestions: {}", String::from_utf8_lossy(&output.stderr));
}
