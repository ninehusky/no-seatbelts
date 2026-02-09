use kernel::hil::gpio;
use kernel::hil::gpio::{Configuration, FloatingState};

pub mod alarm;
pub mod button;
pub mod console;
pub mod led;
pub mod spi_peripheral;
pub mod stream;

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
        Configuration::Input
    }
    fn make_output(&self) -> Configuration {
        Configuration::Input
    }

    fn configuration(&self) -> Configuration {
        Configuration::Input
    }
    fn disable_output(&self) -> Configuration {
        Configuration::Input
    }
    fn disable_input(&self) -> Configuration {
        Configuration::Input
    }
    fn deactivate_to_low_power(&self) {}
    fn set_floating_state(&self, _: FloatingState) {}
    fn floating_state(&self) -> FloatingState {
        FloatingState::PullNone
    }
}
