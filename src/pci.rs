use alloc::vec::*;
use core::{arch::asm, fmt, ptr, slice::from_raw_parts};

/// The PCI Device type.
pub struct PCIDevice {
    pub vendor_id: u32,
    pub device_id: u32,
    pub class_code: u32,
    pub subclass: u32,

    pub bus: u8,
    pub slot: u8,
    pub func: u8,
}

impl PCIDevice {
    /// Creates a new PCI Device type.
    pub fn new(
        vendor_id: u32,
        device_id: u32,
        class_code: u32,
        subclass: u32,
        bus: u8,
        slot: u8,
        func: u8,
    ) -> PCIDevice {
        return PCIDevice {
            vendor_id: vendor_id,
            device_id: device_id,
            class_code: class_code,
            subclass: subclass,
            bus: bus,
            slot: slot,
            func: func,
        };
    }
}

impl fmt::Display for PCIDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vendor ID: {} | Device ID: {} | Class code: {} | Subclass: {} | Bus: {} | Slot: {} | Func: {}",
            self.vendor_id, self.device_id, self.class_code, self.subclass, self.bus, self.slot, self.func
        )
    }
}

/// Writes to a certain PCI device, at a certain offset.
pub fn write_pci(offset: u8, pci_device: &PCIDevice, value: u32) {
    let address = (1 << 31)
        | ((pci_device.bus as u32) << 16)
        | ((pci_device.slot as u32) << 11)  // Slot should be correct
        | ((pci_device.func as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        // Write to PCI address port
        core::arch::asm!("out dx, eax", in("dx") 0xCF8, in("eax") address);

        // Write to PCI data port
        core::arch::asm!("out dx, eax", in("dx") 0xCFC, in("eax") value);
    }
}

/// Reads from a certain PCI device, at a certain offset.
pub fn read_pci(offset: u8, pci_device: &PCIDevice) -> u32 {
    let address = (1 << 31)
        | ((pci_device.bus as u32) << 16)
        | ((pci_device.slot as u32) << 11)  // This should be slot instead of device_id
        | ((pci_device.func as u32) << 8)
        | ((offset as u32) & 0xFC);

    unsafe {
        // Write to PCI address port
        core::arch::asm!("out dx, eax", in("dx") 0xCF8, in("eax") address);

        // Read from PCI data port
        let data: u32;
        core::arch::asm!("in eax, dx", in("dx") 0xCFC, out("eax") data);
        data
    }
}

/// Reads from the PCI config space via the IO ports.
pub unsafe fn read_pci_config(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let addr = (1 << 31)
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);
    asm!("out dx, eax", in("dx") 0xCF8, in("eax") addr);
    let value: u32;
    asm!("in eax, dx", out("eax") value, in("dx") 0xCFC);
    return value;
}

/// Writes to the PCI config space via the IO ports.
pub unsafe fn write_pci_config(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
    let address = (1 << 31)
        | ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC);
    asm!("out dx, eax", in("dx") 0xCF8, in("eax") address);
    asm!("out dx, eax", in("dx") 0xCFC, in("eax") value);
}

/// Scans the PCI bus. Returns a Vec<PCIDevice>
pub fn scan_pci_bus() -> Vec<PCIDevice> {
    let mut devices: Vec<PCIDevice> = Vec::new();
    for bus in 0..255 {
        for slot in 0..32 {
            for func in 0..8 {
                let vendor_id = unsafe { read_pci_config(bus, slot, func, 0x00) } & 0xFFFF;
                if vendor_id != 0xFFFF {
                    let device_id = unsafe { read_pci_config(bus, slot, func, 0x00) >> 16 };
                    let class_code = unsafe { read_pci_config(bus, slot, func, 0x08) >> 24 };
                    let subclass = (unsafe { read_pci_config(bus, slot, func, 0x08) } >> 16) & 0xFF;

                    devices.push(PCIDevice::new(
                        vendor_id, device_id, class_code, subclass, bus, slot, func,
                    ));
                }
            }
        }
    }

    return devices;
}

// !! WARNING !!
//
// PCIe does not work. It's currently broken, and I will eventually fix it.
// It's just an unimplemented function that's pretty important...
// I'll do it eventually.
//
// !! WARNING !!

fn get_ebda_addr() -> u32 {
    let ebda_seg: u16;

    unsafe {
        ebda_seg = *(0x40E as *const u16);
    }

    return (ebda_seg as u32) << 4;
}

fn find_string_in_memory(start_addr: usize, end_addr: usize, target: &str) -> Option<usize> {
    let target_bytes = target.as_bytes();
    let target_len = target_bytes.len();

    let mut addr = start_addr;

    while addr <= end_addr - target_len {
        unsafe {
            // Create a pointer to the current memory address
            let ptr = addr as *const u8;

            // Check if the memory at this address matches the target string
            if from_raw_parts(ptr, target_len) == target_bytes {
                return Some(addr);
            }
        }

        addr += 1;
    }

    None
}

fn get_rsdp() -> [u8; 1024] {
    let ebda_address = get_ebda_addr();
    let mut buffer = [0u8; 1024];
    let mut res_loc: usize;

    unsafe {
        // Read the first 1KB from the EBDA
        let ebda_ptr = ebda_address as *const u8;
        for i in 0..1024 {
            buffer[i] = *ebda_ptr.add(i);
        }
    }

    let rsd_ptr_ = "RSD PTR ";
    let mut found: bool = false;

    for i in 0..=(buffer.len() - rsd_ptr_.as_bytes().len()) {
        if &buffer[i..i + rsd_ptr_.as_bytes().len()] == rsd_ptr_.as_bytes() {
            found = true;
            res_loc = i;
        }
    }

    if found == false {
        let mut addr = 0x000E0000;

        while addr <= 0x000FFFFF - rsd_ptr_.as_bytes().len() {
            unsafe {
                // Create a pointer to the current memory address
                let ptr = addr as *const u8;

                // Check if the memory at this address matches the target string
                if from_raw_parts(ptr, rsd_ptr_.as_bytes().len()) == rsd_ptr_.as_bytes() {
                    found = true;
                    res_loc = addr;
                }
            }

            addr += 1;
        }
    }

    if found == false {
        panic!("(X_X)\n\n[pci]: Unable to get the RSDP.");
    } else {
        return buffer;
    }
}

/// Placeholder for PCIe configuration base retrieval
pub fn get_pcie_config_base() -> usize {
    // Temporary placeholder value
    0x80000000
}

/// Read a 32-bit value from PCIe config space
pub unsafe fn pcie_read(bus: u8, device: u8, function: u8, offset: u16) -> u32 {
    let address = get_pcie_config_base()
        + ((bus as usize) << 20)
        + ((device as usize) << 15)
        + ((function as usize) << 12)
        + (offset as usize);
    ptr::read_volatile(address as *const u32)
}

/// Example PCIe configuration space read function
///
/// Returns in format [vendor_id, device_id, class_code, subclass]
pub fn scan_pcie_bus() -> Vec<[u32; 4]> {
    let mut res_loc: Vec<[u32; 4]> = Vec::new();

    for bus in 0..=255 {
        for device in 0..=31 {
            for function in 0..=7 {
                let vendor_id = unsafe { pcie_read(bus, device, function, 0x00) } & 0xFFFF;
                if vendor_id != 0xFFFF {
                    let device_id = unsafe { pcie_read(bus, device, function, 0x00) >> 16 };
                    let class_code = unsafe { pcie_read(bus, device, function, 0x08) >> 24 };
                    let subclass = (unsafe { pcie_read(bus, device, function, 0x08) } >> 16) & 0xFF;

                    res_loc.push([vendor_id, device_id, class_code, subclass]);
                }
            }
        }
    }

    res_loc
}
