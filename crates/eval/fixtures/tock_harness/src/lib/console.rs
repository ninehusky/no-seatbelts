use capsules_core::console::{App, Console};
use kernel::hil::uart::{Error, ReceiveClient, TransmitClient, UartData};

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_CONSOLE_NEW: fn(
    &'static dyn UartData<'static>,
    &'static mut [u8],
    &'static mut [u8],
    ConsoleGrant,
) -> Console<'static> = console_new_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_CONSOLE_COMMAND: fn(
    &Console<'static>,
    usize,
    usize,
    usize,
    kernel::ProcessId,
) -> kernel::syscall::CommandReturn = console_command_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_CONSOLE_TRANSMITTED_BUFFER: fn(
    &Console<'static>,
    &'static mut [u8],
    usize,
    Result<(), kernel::ErrorCode>,
) = console_transmitted_buffer_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_CONSOLE_RECEIVED_BUFFER: fn(
    &Console<'static>,
    &'static mut [u8],
    usize,
    Result<(), kernel::ErrorCode>,
    Error,
) = console_received_buffer_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_CONSOLE_ALLOCATE_GRANT: fn(
    &Console<'static>,
    kernel::ProcessId,
) -> Result<(), kernel::process::Error> = console_allocate_grant_shim;

#[inline(never)]
fn console_allocate_grant_shim(
    drv: &Console<'static>,
    processid: kernel::ProcessId,
) -> Result<(), kernel::process::Error> {
    kernel::syscall::SyscallDriver::allocate_grant(drv, processid)
}

#[inline(never)]
fn console_received_buffer_shim(
    drv: &Console<'static>,
    buffer: &'static mut [u8],
    rx_len: usize,
    rcode: Result<(), kernel::ErrorCode>,
    error: Error,
) {
    ReceiveClient::received_buffer(drv, buffer, rx_len, rcode, error)
}

#[inline(never)]
fn console_transmitted_buffer_shim(
    drv: &Console<'static>,
    buffer: &'static mut [u8],
    len: usize,
    result: Result<(), kernel::ErrorCode>,
) {
    TransmitClient::transmitted_buffer(drv, buffer, len, result)
}

#[inline(never)]
fn console_command_shim(
    drv: &Console<'static>,
    command_num: usize,
    data: usize,
    a2: usize,
    processid: kernel::ProcessId,
) -> kernel::syscall::CommandReturn {
    kernel::syscall::SyscallDriver::command(drv, command_num, data, a2, processid)
}

type ConsoleGrant = kernel::grant::Grant<
    App,
    kernel::grant::UpcallCount<3>,
    kernel::grant::AllowRoCount<2>,
    kernel::grant::AllowRwCount<2>,
>;

#[inline(never)]
fn console_new_shim(
    uart: &'static dyn UartData<'static>,
    tx_buffer: &'static mut [u8],
    rx_buffer: &'static mut [u8],
    grant: ConsoleGrant,
) -> Console<'static> {
    Console::new(uart, tx_buffer, rx_buffer, grant)
}
