use clap::Parser;

use crate::cli::EvalArgs;

mod analysis;
mod benchmarks;
mod cli;
mod docker;
pub(crate) mod transforms;
mod workspace;

use benchmarks::Benchmark;

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
