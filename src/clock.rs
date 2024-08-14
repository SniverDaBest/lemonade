use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::instructions::port::Port;

// PIT ports
const PIT_COMMAND_PORT: u16 = 0x43;
const PIT_CHANNEL0_PORT: u16 = 0x40;

// Frequency and divisor for PIT
const PIT_FREQUENCY: u32 = 100;
const PIT_DIVISOR: u16 = (1193180 / PIT_FREQUENCY) as u16;

pub struct Timer {
    tick_count: u64,
}

impl Timer {
    pub fn new() -> Self {
        Timer { tick_count: 0 }
    }

    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    pub fn time_elapsed(&self) -> core::time::Duration {
        core::time::Duration::from_millis(self.tick_count as u64 * 10) // Assuming each tick is 10 ms
    }
}

pub fn initialize_pit() {
    unsafe {
        let mut wait_port: Port<u8> = Port::new(0x80);
        let mut wait = || wait_port.write(0); 

        // Set PIT mode
        outb(PIT_COMMAND_PORT, 0x36);

        // Send divisor to PIT
        outb(PIT_CHANNEL0_PORT, (PIT_DIVISOR & 0xFF) as u8);
        outb(PIT_CHANNEL0_PORT, ((PIT_DIVISOR >> 8) & 0xFF) as u8);
    }
}

/// Initialized the interrupt descriptor table
pub fn init_idt(idt: &mut InterruptDescriptorTable) {
    idt[32].set_handler_fn(pit_interrupt_handler);
}

/// PIT Interrupt Handler
extern "x86-interrupt" fn pit_interrupt_handler(_stack_frame: x86_64::structures::idt::InterruptStackFrame) {
    // Increment system time or handle the timer interrupt
}

/// Pauses the clock for some amount of time
pub fn sleep(duration: core::time::Duration) {
    let start = Timer::new();
    while start.time_elapsed() < duration {
        // Busy-wait or sleep
    }
}

/// Helper function to write to ports
unsafe fn outb(port: u16, value: u8) {
    let aligned_port = core::mem::align_of::<u16>() as usize * (port as usize / core::mem::align_of::<u16>() as usize);
    core::ptr::write_volatile(aligned_port as *mut u8, value);
}
