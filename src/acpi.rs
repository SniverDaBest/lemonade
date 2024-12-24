use crate::println;

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
        ptr = unsafe { ptr.add(16) }; // Scan in 16-byte increments
    }
    return None;
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
    let header = unsafe { &*(rsdt_address as *const SDTHeader) };

    // Verify checksum
    let length = header.length as usize;
    let table_bytes =
        unsafe { core::slice::from_raw_parts(header as *const _ as *const u8, length) };

    let checksum = table_bytes
        .iter()
        .copied()
        .fold(0u8, |acc, x| acc.wrapping_add(x));
    if checksum != 0 {
        panic!("(X_X)  Invalid RSDT checksum");
    }

    let entry_count = (length - core::mem::size_of::<SDTHeader>()) / 4;
    let entries = unsafe {
        core::slice::from_raw_parts(
            (header as *const _ as *const u8).add(core::mem::size_of::<SDTHeader>()) as *const u32,
            entry_count,
        )
    };

    for &entry in entries {
        let table_header = unsafe { &*(entry as *const SDTHeader) };
        let signature = core::str::from_utf8(&table_header.signature).unwrap_or("buh?");
        println!("Found ACPI table: {}", signature);
    }
}

#[repr(C, packed)]
pub struct FADT {
    pub header: SDTHeader,
    pub firmware_ctrl: u32,
    pub dsdt: u32,
    pub reserved: u8,
    pub preferred_pm_profile: u8,
    pub sci_int: u16,
    pub smi_cmd: u32,
    pub acpi_enable: u8,
    pub acpi_disable: u8,
}

pub fn parse_fadt(fadt_address: u32) {
    let fadt = unsafe { &*(fadt_address as *const FADT) };
    println!("FADT ACPI Enable: {}", fadt.acpi_enable);
}
