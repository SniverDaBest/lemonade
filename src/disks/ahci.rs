use crate::{
    pci,
    println,
};
use alloc::vec::*;
use core::fmt;

pub struct AHCIController {
    pub pci_device: pci::PCIDevice,
    pub base_addr: u32,
}

pub struct AHCIDevice<'a> {
    pub port: u32,
    pub controller: &'a AHCIController,  // Reference to the AHCI controller this device belongs to
}

impl<'a> AHCIDevice<'a> {
    pub fn new(port: u32, controller: &'a AHCIController) -> AHCIDevice<'a> {
        AHCIDevice { port, controller }
    }
}

impl AHCIController {
    pub fn new(pci_device: pci::PCIDevice, base_addr: u32) -> AHCIController {
        return AHCIController { pci_device: pci_device, base_addr: base_addr };
    }
}

impl fmt::Display for AHCIController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PCI Device: {}", self.pci_device
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
            if verbose { println!("PCI Device with Device ID {}, and Vendor ID {}, is not an AHCI device!", device.device_id, device.vendor_id); };
        }
    }

    return controllers;
}

/// Scans for used AHCI ports inside of an AHCI controller.
pub fn scan_for_used_ports(controller: &AHCIController, verbose: bool) -> Vec<AHCIDevice> {
    let num_ports = 32; // Assume 32 ports for this example
    let devices = check_used_ports(controller, num_ports, verbose);

    if verbose {
        for device in &devices {
            println!("Found device on Controller at base address: {:08x}, Port: {}", controller.base_addr, device.port);
        }
    }

    devices
}


/// Check if each port has a device connected for a given AHCI controller
fn check_used_ports<'a>(controller: &'a AHCIController, num_ports: u32, verbose: bool) -> Vec<AHCIDevice<'a>> {
    let mut res = Vec::new();
    let ahci_base = controller.base_addr;

    for port in 0..num_ports {
        let port_base = ahci_base + (port * 0x80); // Each port is offset by 0x80
        let port_ssts = unsafe { core::ptr::read_volatile((port_base + 0x28) as *const u32) }; // PxSSTS register

        let det_status = port_ssts & 0xF; // Check the device detection status (bits 0-3)
        
        if det_status == 0x3 {
            if verbose { println!("Port {}: Device connected", port); };
            res.push(AHCIDevice::new(port, controller));
        } else {
            if verbose { println!("Port {}: No device connected", port); };
        }
    }

    res
}
