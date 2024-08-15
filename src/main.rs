#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use blog_os::println;
use blog_os::task::{executor::Executor, keyboard, Task};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use blog_os::randomness::Xorshift32;
use blog_os::base64;
use alloc::string::{String, ToString};
use blog_os::command_line::run_command_line;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::allocator;
    use blog_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Lemonade 24m8");
    blog_os::init();

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
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
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
fn randomness() {
    let mut rng = Xorshift32::new(4123);
    
    let expected_values: [u32; 10] = [
        0x426D525A,
        0xECEAAF69,
        0x99FDAEAA,
        0xA937EF7E,
        0xCFD8A752,
        0xBD63D3AB,
        0x25CCD420,
        0x5659FB04,
        0x4E10BC98,
        0x69F19B79,
    ];

    for &expected in &expected_values {
        let result = rng.next();
        assert_eq!(result, expected, "Expected {} but got {}.", expected, result);
    }
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
