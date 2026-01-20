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

use kernel::grant::AllowRoCount;
use kernel::grant::AllowRwCount;
use kernel::grant::UpcallCount;
use kernel::hil::gpio;
use kernel::hil::gpio::Configuration;
use kernel::hil::gpio::FloatingState;
use kernel::hil::gpio::InterruptPin;
use kernel::hil::gpio::{InterruptEdge, InterruptValueWrapper};

// button.rs
// We need to bring in:
// - <Button as InterruptPin>::new
// - <Button as InterruptPin::get_button_state>
// - <Button as SyscallDriver>::command
// - <Button as SyscallDriver>::allocate_grant
// - <Button as ClientWithValue>::fired

type ButtonPins<'a> = &'a [(
    &'a kernel::hil::gpio::InterruptValueWrapper<'a, HarnessPin>,
    kernel::hil::gpio::ActivationMode,
    kernel::hil::gpio::FloatingState,
)];

type ButtonGrant =
    kernel::grant::Grant<crate::button::App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>;

struct HarnessPin;

impl gpio::Input for HarnessPin {
    fn read(&self) -> bool {
        false
    }
}

impl gpio::Output for HarnessPin {
    fn set(&self) {}
    fn clear(&self) {}
    fn toggle(&self) -> bool {
        false
    }
}

impl gpio::Configure for HarnessPin {
    fn make_input(&self) -> Configuration {
        todo!()
    }
    fn make_output(&self) -> Configuration {
        todo!()
    }

    fn configuration(&self) -> Configuration {
        todo!()
    }
    fn disable_output(&self) -> Configuration {
        todo!()
    }
    fn disable_input(&self) -> Configuration {
        todo!()
    }
    fn deactivate_to_low_power(&self) {
        todo!()
    }
    fn set_floating_state(&self, _: FloatingState) {
        todo!()
    }
    fn floating_state(&self) -> FloatingState {
        todo!()
    }
}

impl<'a> gpio::Interrupt<'a> for HarnessPin {
    fn set_client(&self, _client: &'a dyn gpio::Client) {}
    fn enable_interrupts(&self, _something: InterruptEdge) {}
    fn disable_interrupts(&self) {}
    fn is_pending(&self) -> bool {
        false
    }
}

#[inline(never)]
fn button_new_shim(
    pins: ButtonPins<'static>,
    grant: ButtonGrant,
) -> button::Button<'static, HarnessPin> {
    button::Button::new(pins, grant)
}

#[used]
#[unsafe(link_section = ".keep.syms")]
static KEEP_BUTTON_NEW: fn(
    ButtonPins<'static>,
    ButtonGrant,
) -> button::Button<'static, HarnessPin> = button_new_shim;

#[inline(never)]
fn button_command_shim(
    drv: &button::Button<'static, HarnessPin>,
    cmd: usize,
    a1: usize,
    a2: usize,
    pid: ProcessId,
) -> CommandReturn {
    SyscallDriver::command(drv, cmd, a1, a2, pid)
}
#[used]
#[unsafe(link_section = ".keep.syms")]
static KEEP_BUTTON_COMMAND: fn(
    &button::Button<'static, HarnessPin>,
    usize,
    usize,
    usize,
    ProcessId,
) -> CommandReturn = button_command_shim;

#[inline(never)]
fn button_allocate_grant_shim(
    drv: &button::Button<'static, HarnessPin>,
    pid: ProcessId,
) -> Result<(), kernel::process::Error> {
    SyscallDriver::allocate_grant(drv, pid)
}

#[used]
#[unsafe(link_section = ".keep.syms")]
static KEEP_BUTTON_ALLOCATE_GRANT: fn(
    &button::Button<'static, HarnessPin>,
    ProcessId,
) -> Result<(), kernel::process::Error> = button_allocate_grant_shim;

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
        core::ptr::read_volatile(&KEEP_BUTTON_COMMAND);
        core::ptr::read_volatile(&KEEP_BUTTON_ALLOCATE_GRANT);
    }

    loop {}
}
