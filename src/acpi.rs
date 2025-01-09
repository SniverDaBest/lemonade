use x86_64::{instructions::port::Port, structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, Size4KiB}, PhysAddr, VirtAddr};
use crate::{println, serial_println};
use core::isize;

#[repr(C, packed)]
pub struct RSDP {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
}

impl RSDP {
    pub fn validate(&self) -> bool {
        let bytes = unsafe {
            if core::mem::size_of::<Self>() > isize::MAX as usize {
                panic!("(X_X)  Size of RSDP is greater than the max size of an isize.");
            }
            core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of::<Self>())
        };
        bytes
            .iter()
            .copied()
            .fold(0u8, |acc, x| acc.wrapping_add(x))
            == 0
    }
}

#[repr(C, packed)]
pub struct XSDP {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32, // deprecated since 2.0
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    pub reserved: [u8; 3],
}

impl XSDP {
    pub fn validate(&self) -> bool {
        if core::mem::size_of::<Self>() > isize::MAX as usize {
            panic!("(X_X)  Size of XSDP is greater than the max size of an isize.");
        }

        let bytes = unsafe {
            core::slice::from_raw_parts(self as *const _ as *const u8, core::mem::size_of::<Self>())
        };
        bytes
            .iter()
            .copied()
            .fold(0u8, |acc, x| acc.wrapping_add(x))
            == 0
    }
}

pub fn find_rsdp() -> Option<&'static RSDP> {
    let start = 0xE0000 as *const u8;
    let end = 0xFFFFF as *const u8;

    let mut ptr = start;

    while ptr < end {
        if unsafe { core::ptr::read(ptr) } == b'R'
            && unsafe { core::ptr::read(ptr.add(1)) } == b'S'
            && unsafe { core::ptr::read(ptr.add(2)) } == b'D'
            && unsafe { core::ptr::read(ptr.add(3)) } == b' '
            && unsafe { core::ptr::read(ptr.add(4)) } == b'P'
            && unsafe { core::ptr::read(ptr.add(5)) } == b'T'
            && unsafe { core::ptr::read(ptr.add(6)) } == b'R'
            && unsafe { core::ptr::read(ptr.add(7)) } == b' '
        {
            let rsdp = unsafe { &*(ptr as *const RSDP) };
            if rsdp.validate() {
                return Some(rsdp);
            }
        }
        ptr = unsafe { ptr.add(16) };
    }
    return None;
}

pub fn parse_rsdp(rsdp_addr: u32) {
    let rsdp = unsafe { &*(rsdp_addr as *const RSDP) };
    if rsdp.signature != *b"RSD PTR " {
        panic!("(X_X)  Invalid RSDP signature!");
    }
}

pub unsafe fn switch_to_acpi_mode(fadt_addr: u32) {
    let fadt = &*(fadt_addr as *const FADT);
    let smi_cmd = fadt.smi_cmd;
    let acpi_enable = fadt.acpi_enable;

    if smi_cmd == 0 && acpi_enable == 0 && fadt.pm1a_cnt_blk != 0 {
        println!("ACPI is already enabled! Skipping...");
        return;
    }

    // Send the AcpiEnable command to the SMI Command Port
    let mut port = Port::new(smi_cmd as u16);
    port.write(acpi_enable);

    // Check if the ACPI enable command was successful
    if fadt.acpi_disable != 0 {
        panic!("(X_X)  Couldn't enable ACPI!");
    }
}

pub unsafe fn map_acpi_region(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    let start_phys = PhysAddr::new(0xE0000); // Start of ACPI tables (already aligned)
    let end_phys = PhysAddr::new(0x1FFFFF);  // End address, may need to extend if ACPI tables are larger

    for addr in (start_phys.as_u64()..end_phys.as_u64()).step_by(4096) {
        let frame = PhysFrame::containing_address(PhysAddr::new(addr));
        let page = Page::containing_address(VirtAddr::new(addr)); // Virtual address is the same as physical here

        unsafe {
            mapper
                .map_to(page, frame, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator)
                .expect("Page mapping failed!")
                .flush();
        }
    }

    let start_phys2 = PhysAddr::new(0x7FE0000); // Start of ACPI tables (already aligned)
    let end_phys2 = PhysAddr::new(0x8000000);  // End address, may need to extend if ACPI tables are larger

    for addr in (start_phys2.as_u64()..end_phys2.as_u64()).step_by(4096) {
        let frame2 = PhysFrame::containing_address(PhysAddr::new(addr));
        let page2 = Page::containing_address(VirtAddr::new(addr)); // Virtual address is the same as physical here

        unsafe {
            mapper
                .map_to(page2, frame2, PageTableFlags::PRESENT | PageTableFlags::WRITABLE, frame_allocator)
                .expect("Page mapping failed!")
                .flush();
        }
    }
}

#[repr(C, packed)]
pub struct SDTHeader {
    pub signature: [u8; 4],
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

pub fn parse_rsdt(rsdt_address: u32) {
    // Ensure the pointer is non-null and aligned
    if rsdt_address == 0 || rsdt_address % core::mem::align_of::<SDTHeader>() as u32 != 0 {
        panic!("(X_X)  RSDT address is null or not aligned");
    }

    let header = unsafe { &*(rsdt_address as *const SDTHeader) };

    // Verify checksum
    let length = header.length as usize;
    if length > isize::MAX as usize {
        panic!("(X_X)  RSDT length exceeds isize::MAX");
    }

    // Ensure the pointer arithmetic does not overflow
    let header_ptr = header as *const _ as usize;
    let end_ptr = header_ptr.checked_add(length).expect("(X_X)  Pointer arithmetic overflow");

    // Ensure the end pointer is within valid range
    if end_ptr > isize::MAX as usize {
        panic!("(X_X)  End pointer exceeds isize::MAX");
    }

    let table_bytes = unsafe {
        core::slice::from_raw_parts(header as *const _ as *const u8, length)
    };

    let checksum = table_bytes
        .iter()
        .copied()
        .fold(0u8, |acc, x| acc.wrapping_add(x));
    if checksum != 0 {
        panic!("(X_X)  Invalid RSDT checksum");
    }

    let entry_count = (length - core::mem::size_of::<SDTHeader>()) / 4;
    if entry_count > isize::MAX as usize {
        panic!("(X_X)  Entry Count exceeds isize::MAX");
    }

    let entries_ptr = (header as *const _ as *const u8).wrapping_add(core::mem::size_of::<SDTHeader>()) as *const u32;
    if entries_ptr.is_null() || (entries_ptr as usize) % core::mem::align_of::<u32>() != 0 {
        panic!("(X_X)  Entries pointer is null or not aligned");
    }

    let entries = unsafe {
        core::slice::from_raw_parts(entries_ptr, entry_count)
    };

    for &entry in entries {
        let table_header = unsafe { &*(entry as *const SDTHeader) };
        let signature = core::str::from_utf8(&table_header.signature).unwrap_or("buh?");
        println!("Found ACPI table: {}", signature);
    }
}

pub fn find_facp(rsdt_address: u32) -> Option<u32> {
    let header = unsafe { &*(rsdt_address as *const SDTHeader) };

    // Verify checksum
    let length = header.length as usize;
    if length > isize::MAX as usize {
        panic!("(X_X)  SDT Header is greater than an isize!");
    }
    let table_bytes =
        unsafe { core::slice::from_raw_parts(header as *const _ as *const u8, length) };

    let checksum = table_bytes
        .iter()
        .copied()
        .fold(0u8, |acc, x| acc.wrapping_add(x));
    if checksum != 0 {
        panic!("(X_X)  Invalid RSDT checksum");
    }

    let entry_count = (length - core::mem::size_of::<SDTHeader>()) / 8;
    if entry_count > isize::MAX as usize {
        panic!("(X_X)  Entry Count is greater than an isize!");
    }
    let entries = unsafe {
        core::slice::from_raw_parts(
            (header as *const _ as *const u8).add(core::mem::size_of::<SDTHeader>()) as *const u64,
            entry_count,
        )
    };

    for &entry in entries {
        let table_header = unsafe { &*(entry as *const SDTHeader) };
        let signature = core::str::from_utf8(&table_header.signature).unwrap_or("buh?");
        if signature == "FACP" {
            return Some(entry as u32);
        }
    }
    return None;
}


#[repr(C)]
pub struct GenericAddress {
    /// Address space can be one of the following:\
    /// 0: System memory\
    /// 1: System I/O\
    /// 2: PCI configuration space\
    /// 3: Embedded controller\
    /// 4: SMBus\
    /// 5: System CMOS\
    /// 6: Message Signaled Interrupts\
    /// 7: Platform Communication Channel\
    /// 8: Functional Fixed Hardware\
    /// 9: System Firmware\
    /// 0x0A: Platform Communication Channel\
    /// 0x0B - 0x7F: Reserved\
    /// 0x80 - 0xFF: OEM-defined
    pub addr_space: u8,
    /// Only used when needing to access a bitfield
    pub bit_width: u8,
    /// Only used when needing to access a bitfield
    pub bit_offset: u8,
    /// Defines how many bytes you can read/write at once.\
    /// There are 5 possible values:\
    /// 0: Undefined (legacy reasons)\
    /// 1: Byte access\
    /// 2: Word access (16-bit)\
    /// 3: Dword access (32-bit)\
    /// 4: Qword access (64-bit)
    pub access_size: u8,
    /// A 64-bit pointer to defined address space in the data structure
    pub addr: u64,
}

/// Fixed ACPI Description Table\
/// ***Note**: You should preserve the pointer to the FACS in firmware_ctrl (if < 4GB) or x_firmware_ctrl (if >= 4GB)!*
#[repr(C)]
pub struct FADT {
    pub header: SDTHeader,
    /// A 32-bit pointer to the FACS.\
    /// Will be 0 if x_firmware_ctrl isn't.
    pub firmware_ctrl: u32,
    /// A 32-bit pointer to the DSDT.\
    /// Will be 0 if x_dsdt isn't.
    pub dsdt: u32,
    pub reserved: u8,
    /// This has 8 possible values.\
    /// 0: Unspecified\
    /// 1: Desktop\
    /// 2: Mobile\
    /// 3: Workstation\
    /// 4: Enterprise server\
    /// 5: SOHO server\
    /// 6: Aplliance PC\
    /// 7: Performance Server\
    /// >7: Reserved
    pub preferred_pm_profile: u8,
    /// Used to give fixed events (and GPEs) to the OS.\
    /// For example, pushing the power button.\
    /// It will indicate the PIC/IOAPIC interrupt pin for it.
    pub sci_interrupt: u16,
    /// An I/O port. This is where the OS will write AcpiEnable or AcpiDisable to get ownership over the ACPI registers.\
    /// ***Note**: This will be 0 on systems where the System Management Mode isn't supported.*
    pub smi_cmd: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
    pub s4bios_req: u8,
    pub pstate_cnt: u8,
    pub pm1a_evt_blk: u32,
    pub pm1b_evt_blk: u32,
    pub pm1a_cnt_blk: u32,
    pub pm1b_cnt_blk: u32,
    pub pm2_cnt_blk: u32,
    pub pm_tmr_blk: u32,
    pub gpe0_blk: u32,
    pub gpe1_blk: u32,
    pub pm1_evt_len: u8,
    pub pm1_cnt_len: u8,
    pub pm2_cnt_len: u8,
    pub pm_tmr_len: u8,
    pub gpe0_blk_len: u8,
    pub gpe1_blk_len: u8,
    pub gpe1_base: u8,
    pub cst_cnt: u8,
    pub p_lvl2_lat: u16,
    pub p_lvl3_lat: u16,
    pub flush_size: u16,
    pub flush_stride: u16,
    pub duty_offset: u8,
    pub duty_width: u8,
    pub day_alarm: u8,
    pub month_alarm: u8,
    pub century: u8,
    pub iapc_boot_arch: u16,
    pub reserved2: u8,
    pub flags: u32,
    pub reset_reg: GenericAddress,
    pub reset_value: u8,
    pub arm_boot_arch: u16,
    pub fadt_minor_version: u8,
    pub x_firmware_ctrl: u64,
    pub x_dsdt: u64,
    pub x_pm1a_evt_blk: GenericAddress,
    pub x_pm1b_evt_blk: GenericAddress,
    pub x_pm1a_cnt_blk: GenericAddress,
    pub x_pm1b_cnt_blk: GenericAddress,
    pub x_pm2_cnt_blk: GenericAddress,
    pub x_pm_tmr_blk: GenericAddress,
    pub x_gpe0_blk: GenericAddress,
    pub x_gpe1_blk: GenericAddress,
}

pub fn parse_fadt(fadt_addr: u32) {
    if fadt_addr % 4096 != 0 {
        panic!("(X_X)  FADT address is not page-aligned!");
    }
    let fadt = unsafe { &*(fadt_addr as *const FADT) };
    serial_println!("ACPI Version: {}", fadt.header.revision);
    serial_println!("Preferred PM Profile: {}", fadt.preferred_pm_profile);
    serial_println!("SCI Interrupt: {}", fadt.sci_interrupt);
    serial_println!("SMI Command Port: {:#X}", fadt.smi_cmd);
    serial_println!("ACPI Enable: {:#X}", fadt.acpi_enable);
    serial_println!("ACPI Disable: {:#X}", fadt.acpi_disable);
    serial_println!("S4BIOS Request: {:#X}", fadt.s4bios_req);
    serial_println!("P-State Control Register Count: {}", fadt.pstate_cnt);
    serial_println!("PM1A Event Block: {:#X}", fadt.pm1a_evt_blk);
    serial_println!("PM1B Event Block: {:#X}", fadt.pm1b_evt_blk);
    serial_println!("PM1A Control Block: {:#X}", fadt.pm1a_cnt_blk);
    serial_println!("PM1B Control Block: {:#X}", fadt.pm1b_cnt_blk);
    serial_println!("PM2 Control Block: {:#X}", fadt.pm2_cnt_blk);
    serial_println!("PM Timer Block: {:#X}", fadt.pm_tmr_blk);
    serial_println!("GPE0 Block: {:#X}", fadt.gpe0_blk);
    serial_println!("GPE1 Block: {:#X}", fadt.gpe1_blk);
    serial_println!("PM1 Event Length: {}", fadt.pm1_evt_len);
    serial_println!("PM1 Control Length: {}", fadt.pm1_cnt_len);
    serial_println!("PM2 Control Length: {}", fadt.pm2_cnt_len);
    serial_println!("PM Timer Length: {}", fadt.pm_tmr_len);
    serial_println!("GPE0 Block Length: {}", fadt.gpe0_blk_len);
    serial_println!("GPE1 Block Length: {}", fadt.gpe1_blk_len);
    serial_println!("GPE1 Base: {}", fadt.gpe1_base);
    serial_println!("CST Control: {}", fadt.cst_cnt);
    serial_println!("P-State Level 2 Latency: {}", fadt.p_lvl2_lat);
    serial_println!("P-State Level 3 Latency: {}", fadt.p_lvl3_lat);
    serial_println!("Flush Size: {}", fadt.flush_size);
    serial_println!("Flush Stride: {}", fadt.flush_stride);
    serial_println!("Duty Offset: {}", fadt.duty_offset);
    serial_println!("Duty Width: {}", fadt.duty_width);
    serial_println!("Day Alarm: {}", fadt.day_alarm);
    serial_println!("Month Alarm: {}", fadt.month_alarm);
    serial_println!("Century: {}", fadt.century);
    serial_println!("IAPC Boot Architecture: {}", fadt.iapc_boot_arch);
    serial_println!("Flags: {:#X}", fadt.flags);
    serial_println!("Reset Value: {:#X}", fadt.reset_value);
    serial_println!("ARM Boot Architecture: {}", fadt.arm_boot_arch);
    serial_println!("Minor Version: {}", fadt.fadt_minor_version);
    serial_println!("Extended Firmware Control: {:#X}", fadt.x_firmware_ctrl);
    serial_println!("Extended DSDT: {:#X}", fadt.x_dsdt);
}
