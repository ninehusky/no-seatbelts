use kernel::grant::AllowRoCount;
use kernel::grant::AllowRwCount;
use kernel::grant::UpcallCount;
use kernel::hil::gpio;
use kernel::hil::gpio::Configuration;
use kernel::hil::gpio::FloatingState;
use kernel::hil::gpio::InterruptPin;
use kernel::hil::gpio::{InterruptEdge, InterruptValueWrapper};

// Bring modules into scope
use capsules_core::led::LedDriver;
use capsules_core::{alarm, button, console, led, spi_peripheral, stream};
use kernel::ProcessId;
use kernel::hil::gpio::ClientWithValue;
use kernel::syscall::CommandReturn;
use kernel::syscall::SyscallDriver;

use capsules_core::button::Button;
use capsules_core::console::Console;

use crate::lib::HarnessPin;

// button.rs
// We need to bring in:
// - <Button as InterruptPin>::new
// - <Button as InterruptPin::get_button_state>
// - <Button as SyscallDriver>::command
// - <Button as SyscallDriver>::allocate_grant
// - <Button as ClientWithValue>::fired

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_BUTTON_NEW: fn(
    ButtonPins<'static>,
    ButtonGrant,
) -> button::Button<'static, HarnessPin> = button_new_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_BUTTON_GET_BUTTON_STATE: fn(
    &button::Button<'static, HarnessPin>,
    usize,
    ProcessId,
) -> CommandReturn = button_get_button_state_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_BUTTON_COMMAND: fn(
    &button::Button<'static, HarnessPin>,
    usize,
    usize,
    usize,
    ProcessId,
) -> CommandReturn = button_command_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_BUTTON_ALLOCATE_GRANT: fn(
    &button::Button<'static, HarnessPin>,
    ProcessId,
) -> Result<(), kernel::process::Error> = button_allocate_grant_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_BUTTON_FIRED: fn(&button::Button<'static, HarnessPin>, u32) = button_fired_shim;

#[inline(never)]
fn button_fired_shim(drv: &button::Button<'static, HarnessPin>, value: u32) {
    ClientWithValue::fired(drv, value)
}

#[inline(never)]
fn button_get_button_state_shim(
    drv: &button::Button<'static, HarnessPin>,
    data: usize,
    pid: ProcessId,
) -> CommandReturn {
    // `get_button_state` isn't public, so we have to
    // invoke it by running `command` with `command_num = 3`.
    button_command_shim(drv, 3, data, 0, pid)
}

type ButtonPins<'a> = &'a [(
    &'a kernel::hil::gpio::InterruptValueWrapper<'a, HarnessPin>,
    kernel::hil::gpio::ActivationMode,
    kernel::hil::gpio::FloatingState,
)];

type ButtonGrant =
    kernel::grant::Grant<button::App, UpcallCount<1>, AllowRoCount<0>, AllowRwCount<0>>;

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

#[inline(never)]
fn button_command_shim(
    drv: &button::Button<'static, HarnessPin>,
    command_num: usize,
    data: usize,
    a2: usize,
    processid: ProcessId,
) -> CommandReturn {
    SyscallDriver::command(drv, command_num, data, a2, processid)
}

#[inline(never)]
fn button_allocate_grant_shim(
    drv: &button::Button<'static, HarnessPin>,
    pid: ProcessId,
) -> Result<(), kernel::process::Error> {
    SyscallDriver::allocate_grant(drv, pid)
}
