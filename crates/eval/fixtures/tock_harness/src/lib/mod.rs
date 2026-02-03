use kernel::hil::gpio;
use kernel::hil::gpio::{Configuration, FloatingState};

pub mod alarm;
pub mod button;
pub mod console;
pub mod led;
pub mod spi_peripheral;

pub struct HarnessPin;

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
