use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "eval")]
#[command(about = "Evaluate no-seatbelts on a given project", long_about = None)]
pub struct EvalArgs {
    pub src_path: PathBuf,
}
