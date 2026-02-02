use capsules_core::alarm::{AlarmData, AlarmDriver};
use core::cell::Cell;
use kernel::ErrorCode;
use kernel::grant::{AllowRoCount, AllowRwCount, UpcallCount};
use kernel::hil::time::{Alarm, AlarmClient, Freq100MHz, Frequency, Ticks, Ticks32, Time};
use kernel::syscall::SyscallDriver;
use kernel::utilities::cells::OptionalCell;

use core::marker::PhantomData;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_ALARM_NEW: fn(
    &'static HarnessAlarm,
    AlarmGrant,
) -> AlarmDriver<'static, HarnessAlarm> = alarm_new_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_ALARM_COMMAND: fn(
    &AlarmDriver<'static, HarnessAlarm>,
    usize,
    usize,
    usize,
    kernel::ProcessId,
) -> kernel::syscall::CommandReturn = alarm_command_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_ALARM_ALARM: fn(&'static AlarmDriver<'static, HarnessAlarm>) -> () =
    alarm_alarm_shim;

#[inline(never)]
fn alarm_alarm_shim(alarm: &'static AlarmDriver<'static, HarnessAlarm>) {
    alarm.alarm();
}

#[inline(never)]
fn alarm_new_shim(
    alarm: &'static HarnessAlarm,
    grant: AlarmGrant,
) -> AlarmDriver<'static, HarnessAlarm> {
    AlarmDriver::new(alarm, grant)
}

#[inline(never)]
fn alarm_command_shim(
    drv: &AlarmDriver<'static, HarnessAlarm>,
    command_num: usize,
    data: usize,
    a2: usize,
    processid: kernel::ProcessId,
) -> kernel::syscall::CommandReturn {
    SyscallDriver::command(drv, command_num, data, a2, processid)
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
