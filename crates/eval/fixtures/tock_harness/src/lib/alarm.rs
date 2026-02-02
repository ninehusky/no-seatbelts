use capsules_core::alarm::{AlarmData, AlarmDriver};
use core::cell::Cell;
use kernel::ErrorCode;
use kernel::grant::{AllowRoCount, AllowRwCount, UpcallCount};
use kernel::hil::time::{Alarm, AlarmClient, Freq100MHz, Frequency, Ticks, Ticks32, Time};
use kernel::utilities::cells::OptionalCell;

use core::marker::PhantomData;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_ALARM_NEW: fn(
    &'static HarnessAlarm,
    AlarmGrant,
) -> AlarmDriver<'static, HarnessAlarm> = alarm_new_shim;

#[inline(never)]
fn alarm_new_shim(
    alarm: &'static HarnessAlarm,
    grant: AlarmGrant,
) -> AlarmDriver<'static, HarnessAlarm> {
    AlarmDriver::new(alarm, grant)
}

type AlarmGrant = kernel::grant::Grant<
    AlarmData<<MockAlarm<'static, Ticks32, Freq100MHz> as Time>::Ticks>,
    UpcallCount<1>,
    AllowRoCount<0>,
    AllowRwCount<0>,
>;

type HarnessAlarm = MockAlarm<'static, Ticks32, Freq100MHz>;

pub struct MockAlarm<'a, T: Ticks, F: Frequency> {
    current_ticks: Cell<T>,
    client: OptionalCell<&'a dyn AlarmClient>,
    _frequency: PhantomData<F>,
}

impl<'a, T: Ticks, F: Frequency> Time for MockAlarm<'a, T, F> {
    type Frequency = F;
    type Ticks = T;

    fn now(&self) -> Self::Ticks {
        self.current_ticks.get()
    }
}

impl<'a, T: Ticks, F: Frequency> Alarm<'a> for MockAlarm<'a, T, F> {
    fn set_alarm_client(&self, client: &'a dyn AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, _reference: Self::Ticks, _dt: Self::Ticks) {
        unimplemented!()
    }

    fn get_alarm(&self) -> Self::Ticks {
        unimplemented!()
    }

    fn disarm(&self) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn is_armed(&self) -> bool {
        unimplemented!()
    }

    fn minimum_dt(&self) -> Self::Ticks {
        unimplemented!()
    }
}
