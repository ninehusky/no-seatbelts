use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// The benchmark to run.
#[derive(Debug, Clone, ValueEnum)]
pub enum Benchmark {
    /// The ring_buffer_smoketest project which only includes a simple ring buffer implementation and a harness that tests it.
    RingBuffer,
    /// The tock_harness project which is an executable calling code from 6 different Tock capsules.
    TockHarness,
    /// The Tock capsules core crate.
    TockCapsulesCore,
}

#[derive(Debug, Parser)]
#[command(name = "eval")]
#[command(about = "Evaluate no-seatbelts on a given project", long_about = None)]
pub struct EvalArgs {
    #[arg(value_enum, help_heading = "Benchmark Selection")]
    pub benchmark: Benchmark,
    pub src_path: Option<PathBuf>,
}
