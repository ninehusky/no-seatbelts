use clap::Parser;

use crate::cli::Benchmark;
use crate::cli::EvalArgs;

mod analysis;
mod benchmarks;
mod cli;
mod docker;
pub(crate) mod transforms;
mod workspace;

#[allow(dead_code)]
const TARGET: &str = "i686-unknown-linux-gnu";

fn main() -> anyhow::Result<()> {
    let args = EvalArgs::parse();

    match args.benchmark {
        Benchmark::RingBuffer => {
            benchmarks::ring_buffer::run()?;
        }
        _ => unimplemented!(),
    };
    Ok(())
}
