extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

use compiletest::common::Mode;

#[derive(Debug)]
struct Args {
    bless: bool,
}

impl Args {
    fn from_args() -> Self {
        let mut bless = false;
        for arg in std::env::args() {
            if arg == "--bless" {
                bless = true;
            }
        }
        Self { bless }
    }
}

fn run_mode(args: &Args, mode: &'static str) {
    let mut config = compiletest::Config {
        mode: Mode::Ui,
        src_base: PathBuf::from(format!("crates/no-seatbelts/tests/{}", mode)),
        rustc_path: std::env::current_dir()
            .unwrap()
            .join("target/debug/no-seatbelts"),
        target_rustcflags: Some(format!(
            "-L {}/target/debug/deps",
            std::env::current_dir().unwrap().display()
        )),
        bless: args.bless,
        strict_headers: true,
        ..Default::default()
    };
    config.link_deps();
    config.clean_rmeta();
    config.clean_rlib();

    compiletest::run_tests(&config);
}

fn main() {
    let args = Args::from_args();
    run_mode(&args, "ui");
}
