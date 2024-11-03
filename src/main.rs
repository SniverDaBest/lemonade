#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lemonade::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use lemonade::{
    base64, command_line::run_command_line, pci, println, randomness::*, sorting::quicksort, task::{
        executor::Executor,
        keyboard,
        Task
    }, graphics::*,
};
use alloc::borrow::ToOwned;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use alloc::{string::{String, ToString}, vec::Vec};
use core::sync::atomic::{AtomicUsize, Ordering};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use lemonade::allocator;
    use lemonade::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Lemonade 24m11");
    lemonade::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("(X_X)\n\nHeap initialization failed.");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(run_command_line()));
    executor.run();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("(X_X)\n\nUh-oh! Lemonade panicked. Here's some info: {}", info);
    lemonade::hlt_loop();
}

/// This function is called on panic, while testing.
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lemonade::test_panic_handler(info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn basic_math() {
    let mut number = 5;
    number += 5;
    number -= 1;
    number *= 5;
    number /= 5;
    assert_eq!(number, 9);
}

#[test_case]
fn string_modification() {
    let mut str = "sample string #1";
    str = "sample string #2";
    assert_eq!(str, "sample string #2");
}

#[test_case]
fn true_is_true() {
    assert_eq!(true,true);
}

#[test_case]
fn encoding_base64() {
    let input = b"Hello, world!";
    let expected = "SGVsbG8sIHdvcmxkIQ";
    assert_eq!(base64::encode(input), expected);
}

#[test_case]
fn decoding_base64() {
    let input = b"SGVsbG8sIHdvcmxkIQ";
    let expected = "Hello, world!";
    assert_eq!(base64::decode(input), expected.to_string());
}

#[test_case]
fn string_concatenation() {
    let string1 = "Hello";
    let string2 = "World";
    let result = string1.to_owned()+string2;

    assert_eq!(result, "HelloWorld".to_string());
}

#[test_case]
fn str_equals_str() {
    assert_eq!("this is an &str", "this is an &str");
}

#[test_case]
fn str_doesnt_equals_str() {
    assert_ne!("this is an &str.", "this is an &str!");
}
