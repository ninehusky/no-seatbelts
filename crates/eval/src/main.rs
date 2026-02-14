use clap::Parser;

use crate::cli::EvalArgs;

mod analysis;
mod cli;
mod docker;
mod experiments;
pub(crate) mod transforms;
mod workspace;

use experiments::Benchmark;

fn main() -> anyhow::Result<()> {
    let args = EvalArgs::parse();

    match args.benchmark {
        Benchmark::RingBuffer => {
            experiments::ring_buffer::run()?;
        }
        Benchmark::TockHarness => {
            experiments::tock_harness::run()?;
        }
        Benchmark::Survey => {
            experiments::survey::run()?;
        }
        _ => unimplemented!(),
    };
    Ok(())
}
