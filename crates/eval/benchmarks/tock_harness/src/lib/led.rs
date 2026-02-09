use kernel::hil::{
    gpio,
    led::{Led, LedHigh, LedLow},
};

use crate::lib::HarnessPin;

// led.rs
// Calls the available trait methods for both LedLow and LedHigh.
// All of these basically just dispatch to the underlying GPIO pin methods,
// so if `HarnessPin` doesn't panic, neither should anything here.

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDLOW_NEW: fn(&'static HarnessPin) -> LedLow<'static, HarnessPin> =
    ledlow_new_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDLOW_INIT: fn(&'static LedLow<'static, HarnessPin>) -> () = ledlow_init_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDLOW_ON: fn(&'static LedLow<'static, HarnessPin>) -> () = ledlow_on_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDLOW_OFF: fn(&'static LedLow<'static, HarnessPin>) -> () = ledlow_off_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDLOW_TOGGLE: fn(&'static LedLow<'static, HarnessPin>) -> () = ledlow_toggle_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDLOW_READ: fn(&'static LedLow<'static, HarnessPin>) -> () = ledlow_read_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDHIGH_NEW: fn(&'static HarnessPin) -> LedHigh<'static, HarnessPin> =
    ledhigh_new_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDHIGH_INIT: fn(&'static LedHigh<'static, HarnessPin>) -> () = ledhigh_init_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDHIGH_ON: fn(&'static LedHigh<'static, HarnessPin>) -> () = ledhigh_on_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDHIGH_OFF: fn(&'static LedHigh<'static, HarnessPin>) -> () = ledhigh_off_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDHIGH_TOGGLE: fn(&'static LedHigh<'static, HarnessPin>) -> () =
    ledhigh_toggle_shim;

#[used]
#[unsafe(link_section = ".keep.syms")]
pub static KEEP_LEDHIGH_READ: fn(&'static LedHigh<'static, HarnessPin>) -> () = ledhigh_read_shim;

#[inline(never)]
fn ledlow_new_shim(pin: &'static HarnessPin) -> LedLow<'static, HarnessPin> {
    LedLow::new(pin)
}

#[inline(never)]
fn ledlow_init_shim(led: &'static LedLow<'static, HarnessPin>) {
    led.init();
}

#[inline(never)]
fn ledlow_on_shim(led: &'static LedLow<'static, HarnessPin>) {
    led.on();
}

#[inline(never)]
fn ledlow_off_shim(led: &'static LedLow<'static, HarnessPin>) {
    led.off();
}

#[inline(never)]
fn ledlow_toggle_shim(led: &'static LedLow<'static, HarnessPin>) {
    led.toggle();
}

#[inline(never)]
fn ledlow_read_shim(led: &'static LedLow<'static, HarnessPin>) {
    led.read();
}

#[inline(never)]
fn ledhigh_new_shim(pin: &'static HarnessPin) -> LedHigh<'static, HarnessPin> {
    LedHigh::new(pin)
}
#[inline(never)]
fn ledhigh_init_shim(led: &'static LedHigh<'static, HarnessPin>) {
    led.init();
}

#[inline(never)]
fn ledhigh_on_shim(led: &'static LedHigh<'static, HarnessPin>) {
    led.on();
}

#[inline(never)]
fn ledhigh_off_shim(led: &'static LedHigh<'static, HarnessPin>) {
    led.off();
}

#[inline(never)]
fn ledhigh_toggle_shim(led: &'static LedHigh<'static, HarnessPin>) {
    led.toggle();
}

#[inline(never)]
fn ledhigh_read_shim(led: &'static LedHigh<'static, HarnessPin>) {
    led.read();
}
