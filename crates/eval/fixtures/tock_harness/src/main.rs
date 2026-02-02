#![no_std]
#![no_main]

// This is a minimal Tock harness that links in the necessary
// kernel and capsule code to satisfy the linker, but does not
// actually run anything. This is used to test that no-seatbelts
// can successfully build Tock applications without panics.

// The drivers are:
// - Alarm
// - Button
// - Console
// - LED
// - SPI Peripheral
// - Stream

mod lib;

use core::panic::PanicInfo;

use lib::button::KEEP_BUTTON_ALLOCATE_GRANT;
use lib::button::KEEP_BUTTON_COMMAND;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Optional: touch it so even aggressive LTO is less tempted to get clever
    unsafe {
        core::ptr::read_volatile(&KEEP_BUTTON_COMMAND);
        core::ptr::read_volatile(&KEEP_BUTTON_ALLOCATE_GRANT);
    }

    loop {}
}
