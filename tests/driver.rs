extern crate compiletest_rs as compiletest;

use std::path::PathBuf;

use compiletest::common::Mode;

fn run_mode(mode: &'static str) {
    let mut config = compiletest::Config::default();
    config.mode = Mode::Ui;
    config.src_base = PathBuf::from(format!("tests/{}", mode));

    // point to your driver binary, not rustc
    config.rustc_path = std::env::current_dir()
        .unwrap()
        .join("target/debug/no-seatbelts");

    // optional: extra rustc flags for library paths or sysroot
    let mut flags = Vec::new();
    flags.push(format!(
        "-L {}/target/debug/deps",
        std::env::current_dir().unwrap().display()
    ));
    config.target_rustcflags = Some(flags.join(" "));

    config.clean_rmeta();
    config.bless = true;
    config.clean_rlib();
    config.strict_headers = true;

    compiletest::run_tests(&config);
}

#[test]
fn compile_test() {
    // run_mode("compile-fail");
    run_mode("ui");
}
