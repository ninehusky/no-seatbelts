use clap::ValueEnum;

pub mod ring_buffer;
pub mod survey;
pub mod tock_board;
pub mod tock_harness;

#[derive(Debug, Clone, ValueEnum)]
pub enum Benchmark {
    /// The ring_buffer_smoketest project which only includes a simple ring buffer implementation and a harness that tests it.
    RingBuffer,
    /// The full nordic nrf52840 Tock board.
    TockBoard,
    /// The tock_harness project which is an executable calling code from 6 different Tock capsules.
    TockHarness,
    /// The Tock capsules core crate.
    TockCapsulesCore,
    /// A series of representative Rust crates.
    Survey,
}
