#![no_std]
#![no_main]

use ring_buffer_smoketest::collections::queue::Queue;
use ring_buffer_smoketest::collections::ring_buffer::RingBuffer;
use core::hint::black_box;
use core::panic::PanicInfo;

// This function is called on panic
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

macro_rules! harness_fn {
    ($name:ident, $body:expr) => {
        #[no_mangle]
        pub fn $name(buf: &mut RingBuffer<i32>) {
            $body(buf);
        }
    };
}

// Define all the wrappers
harness_fn!(call_available_len, |buf: &mut RingBuffer<i32>| {
    black_box(buf.available_len());
});

harness_fn!(call_as_slices, |buf: &mut RingBuffer<i32>| {
    black_box(buf.as_slices());
});

harness_fn!(call_has_elements, |buf: &mut RingBuffer<i32>| {
    black_box(buf.has_elements());
});

harness_fn!(call_is_full, |buf: &mut RingBuffer<i32>| {
    black_box(buf.is_full());
});

harness_fn!(call_len, |buf: &mut RingBuffer<i32>| {
    black_box(buf.len());
});

harness_fn!(call_enqueue, |buf: &mut RingBuffer<i32>| {
    black_box(buf.enqueue(black_box(1)));
});

harness_fn!(call_dequeue, |buf: &mut RingBuffer<i32>| {
    black_box(buf.dequeue());
});

harness_fn!(call_push, |buf: &mut RingBuffer<i32>| {
    black_box(buf.push(black_box(2)));
});

harness_fn!(call_remove_first_matching, |buf: &mut RingBuffer<i32>| {
    let _ = buf.enqueue(3);
    black_box(buf.remove_first_matching(|&x: &i32| x == 3));
});

harness_fn!(call_retain, |buf: &mut RingBuffer<i32>| {
    black_box(buf.retain(|&x: &i32| black_box(x) % black_box(2) == 0));
});

harness_fn!(call_empty, |buf: &mut RingBuffer<i32>| {
    black_box(buf.empty());
});

// A simple function that just returns 3
#[no_mangle]
pub extern "C" fn foo() -> i32 {
    3
}

#[no_mangle]
pub extern "C" fn main() -> !{
    const LEN: usize = 5;
    let mut storage = [0; LEN];
    let mut buf = RingBuffer::new(&mut storage);

    call_available_len(&mut buf);
    call_as_slices(&mut buf);
    call_has_elements(&mut buf);
    call_is_full(&mut buf);
    call_len(&mut buf);
    call_enqueue(&mut buf);
    call_dequeue(&mut buf);
    call_push(&mut buf);
    call_remove_first_matching(&mut buf);
    call_retain(&mut buf);
    call_empty(&mut buf);
    loop {}
}


#[link(name="c")]
extern "C" {
}

#[no_mangle]
pub static TEST_FUNCS: [fn(&mut RingBuffer<i32>); 11] = [
    call_available_len,
    call_as_slices,
    call_has_elements,
    call_is_full,
    call_len,
    call_enqueue,
    call_dequeue,
    call_push,
    call_remove_first_matching,
    call_retain,
    call_empty,
];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    main();
}

// For arm to not complain
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr0() {}
#[no_mangle]
pub extern "C" fn __aeabi_unwind_cpp_pr1() {}
