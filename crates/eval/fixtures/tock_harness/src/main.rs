#![no_std]
#![no_main]

use core::panic::PanicInfo;

// Bring modules into scope
use capsules_core::led::LedDriver;
use capsules_core::{alarm, button, console, led, spi_peripheral, stream};
use kernel::ProcessId;
use kernel::syscall::CommandReturn;
use kernel::syscall::SyscallDriver;

use capsules_core::button::Button;
use capsules_core::console::Console;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

use kernel::hil::gpio;

#[inline(never)]
fn console_command_shim(
    drv: &console::Console<'static>,
    cmd: usize,
    a1: usize,
    a2: usize,
    pid: ProcessId,
) -> CommandReturn {
    SyscallDriver::command(drv, cmd, a1, a2, pid)
}

#[used]
#[unsafe(link_section = ".keep.syms")]
static KEEP_CONSOLE_COMMAND: fn(
    &console::Console<'static>,
    usize,
    usize,
    usize,
    ProcessId,
) -> CommandReturn = console_command_shim;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Optional: touch it so even aggressive LTO is less tempted to get clever
    unsafe {
        core::ptr::read_volatile(&KEEP_CONSOLE_COMMAND);
    }
    loop {}
}
