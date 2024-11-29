use crate::{
    allocator::{
        fixed_size_block::{get_allocator_instance, FixedSizeBlockAllocator},
        Locked,
    },
    pci, println,
};
use alloc::vec::Vec;
use core::{alloc::Layout, fmt};

pub struct AHCIController {
    pub pci_device: pci::PCIDevice,
    pub base_addr: u32,
}

pub struct AHCIDevice<'a> {
    pub port: u32,
    pub controller: &'a AHCIController, // Reference to the AHCI controller this device belongs to
}

#[repr(C, packed)]
struct HBACommandHeader {
    // Command header fields
    cfl: u8,    // Command FIS length
    a: u8,      // ATAPI flag
    w: u8,      // Write flag
    prdtl: u16, // Physical Region Descriptor Table length
    prdbc: u32, // Physical Region Descriptor Byte Count
    ctba: u32,  // Command Table Base Address
    ctbau: u32, // Command Table Base Address Upper 32 bits
    reserved: [u32; 4],
}

#[repr(C, packed)]
struct HBACommandTable {
    cfis: [u8; 64],                 // Command FIS
    acmd: [u8; 16],                 // ATAPI command (if used)
    reserved: [u8; 48],             // Reserved area
    prdt_entry: [HBA_PRDTEntry; 1], // PRDT entries
}

#[repr(C, packed)]
struct HBA_PRDTEntry {
    dba: u32,  // Data Base Address
    dbau: u32, // Upper 32 bits of address
    reserved: u32,
    dbc: u32, // Byte count (interrupt upon completion)
}

impl<'a> AHCIDevice<'a> {
    pub fn new(port: u32, controller: &'a AHCIController) -> AHCIDevice<'a> {
        AHCIDevice { port, controller }
    }
}

impl AHCIController {
    pub fn new(pci_device: pci::PCIDevice, base_addr: u32) -> AHCIController {
        AHCIController {
            pci_device,
            base_addr,
        }
    }

    pub fn read(&self, device: &AHCIDevice, sector: u64, count: u16) -> Vec<u8> {
        // Step 1: Set up command list and FIS structures
        let port = device.port as u32;
        let clb = self.base_addr + (port * 0x80); // Command List Base Address
        let fb = clb + 0x40; // FIS Base Address

        // Allocate a command header, table, and PRDT entry
        let cmd_header = HBACommandHeader {
            cfl: 5, // FIS length = 5 DWORDs for a read command
            a: 0,
            w: 0,     // Read operation
            prdtl: 1, // One PRDT entry for simplicity
            prdbc: 0,
            ctba: fb + 0x80,
            ctbau: 0,
            reserved: [0; 4],
        };

        let prdt_entry = HBA_PRDTEntry {
            dba: fb, // Buffer address (simplified for now)
            dbau: 0,
            reserved: 0,
            dbc: (512 * count as u32) - 1, // 512 bytes per sector
        };

        let mut cmd_table = HBACommandTable {
            cfis: [0; 64],
            acmd: [0; 16],
            reserved: [0; 48],
            prdt_entry: [prdt_entry],
        };

        // Step 2: Set up the command FIS (First Information Structure)
        // The FIS will contain the ATA READ SECTORS command
        cmd_table.cfis[0] = 0x27; // FIS type: RegH2D (Host to Device)
        cmd_table.cfis[1] = 1 << 7; // Command, not control
        cmd_table.cfis[2] = 0x25; // Command: READ DMA EXT

        // Set the starting LBA (Logical Block Address)
        cmd_table.cfis[4] = (sector & 0xFF) as u8;
        cmd_table.cfis[5] = ((sector >> 8) & 0xFF) as u8;
        cmd_table.cfis[6] = ((sector >> 16) & 0xFF) as u8;
        cmd_table.cfis[7] = ((sector >> 24) & 0xFF) as u8;

        cmd_table.cfis[8] = ((sector >> 32) & 0xFF) as u8;
        cmd_table.cfis[9] = ((sector >> 40) & 0xFF) as u8;

        // Set the sector count
        cmd_table.cfis[12] = (count & 0xFF) as u8;
        cmd_table.cfis[13] = ((count >> 8) & 0xFF) as u8;

        // Step 3: Issue the command
        unsafe {
            let port_cmd = (self.base_addr + 0x118 + (port * 0x80)) as *mut u32;
            *port_cmd |= 1 << 0; // Start the command
        }

        // Step 4: Wait for completion
        loop {
            let port_tfd = unsafe {
                core::ptr::read_volatile((self.base_addr + 0x120 + (port * 0x80)) as *const u32)
            };
            if port_tfd & (1 << 7) == 0 {
                break; // Wait until busy bit is cleared
            }
        }

        // Step 5: Read data from the buffer (simplified here)
        let mut buffer = Vec::with_capacity(512 * count as usize);
        unsafe {
            let data_ptr = fb as *const u8;
            core::ptr::copy_nonoverlapping(data_ptr, buffer.as_mut_ptr(), 512 * count as usize);
            buffer.set_len(512 * count as usize);
        }

        buffer
    }
    pub fn write(&self, device: &AHCIDevice, sector: u64, count: u16, data: &[u8]) -> bool {
        // Step 1: Set up command list and FIS structures
        let port = device.port as u32;
        let clb = self.base_addr + (port * 0x80); // Command List Base Address
        let fb = clb + 0x40; // FIS Base Address

        // Allocate a command header, table, and PRDT entry
        let cmd_header = HBACommandHeader {
            cfl: 5, // FIS length = 5 DWORDs for a write command
            a: 0,
            w: 1,     // Write operation
            prdtl: 1, // One PRDT entry for simplicity
            prdbc: 0,
            ctba: fb + 0x80,
            ctbau: 0,
            reserved: [0; 4],
        };

        let prdt_entry = HBA_PRDTEntry {
            dba: fb, // Buffer address
            dbau: 0,
            reserved: 0,
            dbc: (512 * count as u32) - 1, // 512 bytes per sector
        };

        let mut cmd_table = HBACommandTable {
            cfis: [0; 64],
            acmd: [0; 16],
            reserved: [0; 48],
            prdt_entry: [prdt_entry],
        };

        // Step 2: Set up the command FIS (First Information Structure)
        // The FIS will contain the ATA WRITE SECTORS command
        cmd_table.cfis[0] = 0x27; // FIS type: RegH2D (Host to Device)
        cmd_table.cfis[1] = 1 << 7; // Command, not control
        cmd_table.cfis[2] = 0x35; // Command: WRITE DMA EXT

        // Set the starting LBA (Logical Block Address)
        cmd_table.cfis[4] = (sector & 0xFF) as u8;
        cmd_table.cfis[5] = ((sector >> 8) & 0xFF) as u8;
        cmd_table.cfis[6] = ((sector >> 16) & 0xFF) as u8;
        cmd_table.cfis[7] = ((sector >> 24) & 0xFF) as u8;
        cmd_table.cfis[8] = ((sector >> 32) & 0xFF) as u8;
        cmd_table.cfis[9] = ((sector >> 40) & 0xFF) as u8;

        // Set the sector count
        cmd_table.cfis[12] = (count & 0xFF) as u8;
        cmd_table.cfis[13] = ((count >> 8) & 0xFF) as u8;

        // Step 3: Copy data to buffer
        unsafe {
            let data_ptr = data.as_ptr();
            let buf_ptr = fb as *mut u8;
            core::ptr::copy_nonoverlapping(data_ptr, buf_ptr, data.len());
        }

        // Step 4: Issue the command
        unsafe {
            let port_cmd = (self.base_addr + 0x118 + (port * 0x80)) as *mut u32;
            *port_cmd |= 1 << 0; // Start the command
        }

        // Step 5: Wait for completion
        loop {
            let port_tfd = unsafe {
                core::ptr::read_volatile((self.base_addr + 0x120 + (port * 0x80)) as *const u32)
            };
            if port_tfd & (1 << 7) == 0 {
                break; // Wait until busy bit is cleared
            }
        }

        true // Return success
    }
}

impl fmt::Display for AHCIController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PCI Device: {} | Base Address: {}",
            self.pci_device, self.base_addr
        )
    }
}

/// Scans for AHCI controllers and checks their ports for connected devices
pub fn scan_for_ahci_controllers(verbose: bool) -> Vec<AHCIController> {
    let devices = pci::scan_pci_bus(); // Scan the PCI bus for devices
    let mut controllers = Vec::new();

    for device in devices {
        let class_code = device.class_code;
        let subclass = device.subclass;

        // Check if it's an AHCI controller
        if class_code == 0x01 && subclass == 0x06 {
            let bus: u8 = device.bus;
            let slot: u8 = device.slot;
            let func: u8 = device.func;

            // Read the BAR5 (Base Address Register 5) for the AHCI base address
            let bar5 = pci::read_pci(0x24, &device);
            let ahci_base = bar5 & 0xFFFFFFF0; // Mask to get the base address

            if verbose {
                println!(
                    "Found AHCI controller: PCI Device: {} | AHCI Base Address: {:08x}",
                    device, ahci_base
                );
            }

            // Create an AHCIController and push it to the vector
            let controller = AHCIController::new(device, ahci_base);
            controllers.push(controller);
        } else {
            if verbose {
                println!(
                    "PCI Device with Device ID {}, and Vendor ID {}, is not an AHCI device!",
                    device.device_id, device.vendor_id
                );
            };
        }
    }

    return controllers;
}

/// Scans for used AHCI ports inside of an AHCI controller.
pub fn scan_for_used_ports(controller: &AHCIController, verbose: bool) -> Vec<AHCIDevice> {
    let num_ports = 32; // Assume 32 ports for this example
    let mut devices: Vec<AHCIDevice<'_>> = Vec::new();
    let ahci_base = controller.base_addr;

    for port in 0..num_ports {
        let port_base = ahci_base + 0x100 + (port * 0x80);
        let ssts = unsafe { core::ptr::read_volatile((port_base + 0x28) as *const u32) };
        let device_detected = (ssts & 0xF) == 0x3;

        if device_detected {
            if verbose {
                println!("Device detected on port {}", port);
            }
            devices.push(AHCIDevice::new(port, controller));
        } else {
            if verbose {
                println!("No device detected on port {}", port);
            }
        }
    }

    if verbose {
        for device in &devices {
            println!(
                "Found device on Controller at base address: {:08x}, Port: {}",
                controller.base_addr, device.port
            );
        }
    }

    devices
}

/// Easy(ish) way to read sectors from an AHCI device.
/// NOTE: I will *eventually* optimize this to use the count part of `controller.read()`.
pub fn ahci_read(device: &AHCIDevice, sectors: Vec<u64>) -> Vec<u8> {
    let controller = device.controller;
    let mut res: Vec<u8> = Vec::new();
    for sector in sectors {
        res.push(controller.read(&device, sector, 1)[0]);
    }

    return res;
}

pub fn ahci_write(device: &AHCIDevice, sectors: Vec<u64>, data: &[u8]) {
    let controller = device.controller;
    for sector in sectors {
        controller.write(device, sector, 1, data);
    }
}
