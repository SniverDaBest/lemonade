use core::arch::asm;
use alloc::vec::*;

/// Reads from the PCI config space via the IO ports.
pub unsafe fn read_pci(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let addr = (1 << 31) | ((bus as u32) << 16) | ((slot as u32) << 11) | ((func as u32) << 8) | ((offset as u32) & 0xFC);
    asm!("out dx, eax", in("dx") 0xCF8, in("eax") addr);
    let value: u32;
    asm!("in eax, dx", out("eax") value, in("dx") 0xCFC);
    return value;
}

/// Writes to the PCI config space via the IO ports.
pub unsafe fn write_pci(bus: u8, slot: u8, func: u8, offset: u8, value: u32) {
    let address = (1 << 31) | ((bus as u32) << 16) | ((slot as u32) << 11) | ((func as u32) << 8) | ((offset as u32) & 0xFC);
    asm!("out dx, eax", in("dx") 0xCF8, in("eax") address);
    asm!("out dx, eax", in("dx") 0xCFC, in("eax") value);
}

/// Scans the PCI bus.
/// 
/// Returns in format [vendor_id, device_id, class_code, subclass]
pub fn scan_pci_bus() -> Vec<[u32; 4]> {
    let mut res: Vec<[u32; 4]> = Vec::<[u32; 4]>::new();
    for bus in 0..255 {
        for slot in 0..32 {
            for func in 0..8 {
                let vendor_id = unsafe { read_pci(bus, slot, func, 0x00) } & 0xFFFF;
                if vendor_id != 0xFFFF {
                    let device_id = unsafe { read_pci(bus, slot, func, 0x00) >> 16 };
                    let class_code = unsafe { read_pci(bus, slot, func, 0x08) >> 24 };
                    let subclass = (unsafe { read_pci(bus, slot, func, 0x08) } >> 16 ) & 0xFF;

                    res.push([vendor_id, device_id, class_code, subclass]);
                }
            }
        }
    }

    return res;
}