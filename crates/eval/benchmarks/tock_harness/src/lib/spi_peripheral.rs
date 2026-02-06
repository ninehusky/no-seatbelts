use capsules_core::spi_peripheral::{PeripheralApp, SpiPeripheral};
use kernel::ErrorCode;
use kernel::grant::{AllowRoCount, AllowRwCount, Grant, UpcallCount};
use kernel::hil::spi::SpiSlaveClient;
use kernel::hil::spi::{ClockPhase, ClockPolarity, SpiSlaveDevice};

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_SPI_PERIPHERAL_NEW: fn(
    &'static MockSpiSlaveDevice,
    SpiPeripheralGrant,
) -> SpiPeripheral<'static, MockSpiSlaveDevice> = spi_peripheral_new_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_SPI_PERIPHERAL_CONFIG_BUFFERS: fn(
    &SpiPeripheral<'static, MockSpiSlaveDevice>,
    &'static mut [u8],
    &'static mut [u8],
) -> () = spi_peripheral_config_buffers;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_SPI_PERIPHERAL_READ_WRITE_DONE: fn(
    &SpiPeripheral<'static, MockSpiSlaveDevice>,
    Option<&'static mut [u8]>,
    Option<&'static mut [u8]>,
    usize,
    Result<(), kernel::ErrorCode>,
) -> () = spi_peripheral_read_write_done_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_SPI_PERIPHERAL_ALLOCATE_GRANT: fn(
    &SpiPeripheral<'static, MockSpiSlaveDevice>,
    kernel::ProcessId,
) -> Result<(), kernel::process::Error> = spi_peripheral_allocate_grant_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_SPI_PERIPHERAL_CHIP_SELECTED: fn(
    &SpiPeripheral<'static, MockSpiSlaveDevice>,
) -> () = spi_peripheral_chip_selected_shim;

#[inline(never)]
fn spi_peripheral_chip_selected_shim(drv: &SpiPeripheral<'static, MockSpiSlaveDevice>) {
    SpiSlaveClient::chip_selected(drv)
}

#[inline(never)]
fn spi_peripheral_allocate_grant_shim(
    drv: &SpiPeripheral<'static, MockSpiSlaveDevice>,
    processid: kernel::ProcessId,
) -> Result<(), kernel::process::Error> {
    kernel::syscall::SyscallDriver::allocate_grant(drv, processid)
}

#[inline(never)]
fn spi_peripheral_read_write_done_shim(
    drv: &SpiPeripheral<'static, MockSpiSlaveDevice>,
    read_buffer: Option<&'static mut [u8]>,
    write_buffer: Option<&'static mut [u8]>,
    length: usize,
    status: Result<(), kernel::ErrorCode>,
) {
    SpiSlaveClient::read_write_done(drv, read_buffer, write_buffer, length, status)
}

#[inline(never)]
fn spi_peripheral_config_buffers(
    drv: &SpiPeripheral<'static, MockSpiSlaveDevice>,
    read_buffer: &'static mut [u8],
    write_buffer: &'static mut [u8],
) {
    SpiPeripheral::config_buffers(drv, read_buffer, write_buffer);
}

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_SPI_PERIPHERAL_COMMAND: fn(
    &SpiPeripheral<'static, MockSpiSlaveDevice>,
    usize,
    usize,
    usize,
    kernel::ProcessId,
) -> kernel::syscall::CommandReturn = spi_peripheral_command_shim;

type SpiPeripheralGrant =
    kernel::grant::Grant<PeripheralApp, UpcallCount<2>, AllowRoCount<1>, AllowRwCount<1>>;

#[inline(never)]
fn spi_peripheral_new_shim(
    spi_peripheral: &'static MockSpiSlaveDevice,
    grant: SpiPeripheralGrant,
) -> SpiPeripheral<'static, MockSpiSlaveDevice> {
    SpiPeripheral::new(spi_peripheral, grant)
}

#[inline(never)]
fn spi_peripheral_command_shim(
    drv: &SpiPeripheral<'static, MockSpiSlaveDevice>,
    command_num: usize,
    a2: usize,
    data: usize,
    processid: kernel::ProcessId,
) -> kernel::syscall::CommandReturn {
    kernel::syscall::SyscallDriver::command(drv, command_num, a2, data, processid)
}

struct MockSpiSlaveDevice;

impl<'a> SpiSlaveDevice<'a> for MockSpiSlaveDevice {
    fn set_client(&self, _client: &'a dyn kernel::hil::spi::SpiSlaveClient) {}

    fn read_write_bytes(
        &self,
        _write_buffer: Option<&'static mut [u8]>,
        _read_buffer: Option<&'static mut [u8]>,
        _len: usize,
    ) -> Result<
        (),
        (
            kernel::ErrorCode,
            Option<&'static mut [u8]>,
            Option<&'static mut [u8]>,
        ),
    > {
        Ok(())
    }

    fn configure(&self, _: ClockPolarity, _: ClockPhase) -> Result<(), kernel::ErrorCode> {
        Ok(())
    }
    fn set_polarity(&self, _: ClockPolarity) -> Result<(), kernel::ErrorCode> {
        Ok(())
    }
    fn get_polarity(&self) -> ClockPolarity {
        ClockPolarity::IdleLow
    }
    fn set_phase(&self, _: ClockPhase) -> Result<(), kernel::ErrorCode> {
        Ok(())
    }
    fn get_phase(&self) -> ClockPhase {
        ClockPhase::SampleTrailing
    }
}
