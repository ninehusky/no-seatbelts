use clap::Parser;
use std::path::PathBuf;

use crate::benchmarks::Benchmark;

#[derive(Debug, Parser)]
#[command(name = "eval")]
#[command(about = "Evaluate no-seatbelts on a given project", long_about = None)]
pub struct EvalArgs {
    #[arg(value_enum, help_heading = "Benchmark Selection")]
    pub benchmark: Benchmark,
    pub src_path: Option<PathBuf>,
}
