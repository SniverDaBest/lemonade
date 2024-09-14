#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lemonade::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use lemonade::{
    task::{
        executor::Executor,
        keyboard,
        Task
    },
    randomness::*, 
    command_line::run_command_line,
    base64,
    println,
    pci,
};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use alloc::{string::{String, ToString}, vec::Vec};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use lemonade::allocator;
    use lemonade::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Lemonade 24m9");
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

// BROKEN TESTS

/*
#[test_case]
fn scanning_pci_bus() {
    let res = pci::scan_pci_bus();
    let mut exp: Vec<[u32; 4]> = Vec::new();
    exp.push([32902, 4663, 6, 0]);
    exp.push([32902, 28672, 6, 1]);
    exp.push([32902, 28688, 1, 1]);
    exp.push([32902, 28947, 6, 128]);
    exp.push([4660, 4369, 3, 0]);
    exp.push([32902, 4110, 2, 0]);

    assert_eq!(exp, res);
}
*/

/*
#[test_case]
fn test_rand_u16() {
    match rand_u16() {
        Ok(Some(val)) => {
            assert!(val < u16::MAX, "(X_X)  rand_u16 value should be less than u16::MAX");
        },
        Ok(None) => panic!("(X_X)  rand_u16 should return Some value, not None"),
        Err(e) => panic!("(X_X)  {}", e),
    }
}

#[test_case]
fn test_rand_u32() {
    match rand_u32() {
        Ok(Some(val)) => {
            assert!(val < u32::MAX, "(X_X)  rand_u32 value should be less than u32::MAX");
        },
        Ok(None) => panic!("(X_X)  rand_u32 should return Some value, not None"),
        Err(e) => panic!("(X_X)  {}", e),
    }
}

#[test_case]
fn test_rand_u64() {
    match rand_u64() {
        Ok(Some(val)) => {
            assert!(val < u64::MAX, "(X_X)  rand_u64 value should be less than u64::MAX");
        },
        Ok(None) => panic!("(X_X)  rand_u64 should return Some value, not None"),
        Err(e) => panic!("(X_X)  {}", e),
    }
}
*/