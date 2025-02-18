#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![feature(const_mut_refs)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use core::panic::PanicInfo;
use multiboot2::{BootInformation, BootInformationHeader};

pub mod acpi;
pub mod allocator;
pub mod base64;
pub mod cmos;
pub mod command_line;
pub mod disks;
pub mod fs;
pub mod gdt;
pub mod hashmaps;
pub mod interrupts;
pub mod memory;
pub mod pci;
pub mod randomness;
pub mod serial;
pub mod sorting;
pub mod spinlock;
pub mod task;
pub mod vga_buffer;

#[macro_export]
macro_rules! entry_point {
    ($path:path, boot_info: BootInformation) => {
        #[export_name = "_start"]
        pub extern "C" fn _kernel_start(mb_magic: u32, mbi_ptr: u32) -> ! {
            if mb_magic != multiboot2::MAGIC { panic!("(X_X)  Multiboot2 magic given is incorrect!"); }
            let boot_info = BootInformation::load(mbi_ptr as *const BootInformationHeader);
            $path(boot_info)
        }
    };
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[OK]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[FAILED]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
use x86_64::structures::idt::InterruptDescriptorTable;

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main() -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
