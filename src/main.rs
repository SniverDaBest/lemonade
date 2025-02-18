#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lemonade::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use alloc::{
    borrow::ToOwned,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use multiboot2::{BootInformation, BootInformationHeader};
use core::{panic::PanicInfo, ops::Deref};
use lemonade::{
    acpi,
    base64,
    cmos::*,
    command_line::run_command_line,
    println,
    sorting::quicksort,
    task::{executor::Executor, Task},
};

entry_point!(kernel_main);

fn kernel_main(boot_info: BootInformation) -> ! {
    use lemonade::allocator;
    use lemonade::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Lemonade 25m2"); // this should ALWAYS print when booting up. if it doesn't, something's VERY fucked.
    lemonade::init();

    let cmd = boot_info.command_line_tag();

    let time = Time::from_current();
    println!("Current time is: {}", time);

    let phys_mem_offset = VirtAddr::new(
        *boot_info.physical_memory_offset.as_ref().unwrap()
    );

    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mem_map: &'static [MemoryRegion] = boot_info.memory_regions.deref(); // Use the deref() method
    let mut frame_allocator: BootInfoFrameAllocator = unsafe { BootInfoFrameAllocator::init(mem_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("(X_X)\n\nHeap initialization failed.");

    unsafe {
        acpi::map_acpi_region(&mut mapper, &mut frame_allocator);
    }


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
    println!(
        "(X_X)\n\nUh-oh! Lemonade panicked. Here's some info: {}",
        info
    );
    lemonade::hlt_loop();
}

// TEST CODE STARTS HERE.

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
    let mut s = "sample string #1";
    s = "sample string #2";
    assert_eq!(s, "sample string #2");
}

#[test_case]
fn true_is_true() {
    assert_eq!(true, true);
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
    let result = string1.to_owned() + string2;

    assert_eq!(result, "HelloWorld".to_string());
}

#[test_case]
fn str_equals_str() {
    assert_eq!("this is an &str", "this is an &str");
}

#[test_case]
fn str_doesnt_equal_str() {
    assert_ne!("this is an &str.", "this is an &str!");
}
