use std::any;
/// This is a teeny Rust script which evaluates no-seatbelts on a given project.
/// The project has to be `ring_buffer_smoketest` for now.
use std::fs::{self};
use std::path::PathBuf;

use clap::Parser;

use crate::cli::Benchmark;
use crate::docker::{docker_compile, ensure_docker_image};
use crate::{cli::EvalArgs, workspace::copy_dir_recursive};

mod analysis;
mod benchmarks;
mod cli;
mod docker;
pub(crate) mod transforms;
mod workspace;

#[allow(dead_code)]
const TARGET: &str = "i686-unknown-linux-gnu";

// This is for Github Actions compatibility for filenames.
fn sanitize_for_path(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '|' | '*' | '?' | '\n' | '\r' => '_',
            _ => c,
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let repo_root = std::env::current_dir().expect("failed to get current dir");
    let args = EvalArgs::parse();

    match args.benchmark {
        Benchmark::RingBuffer => {
            benchmarks::ring_buffer::run()?;
        }
        _ => unimplemented!(),
    };
    Ok(())
}
