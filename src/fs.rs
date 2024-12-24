use alloc::{string::*, vec, vec::*};
use core::{arch::asm, convert::TryInto};
const FAT32_CLEAN_SHUT_MASK: u32 = 0x08000000;
const FAT32_HARD_ERROR_MASK: u32 = 0x04000000;

#[repr(C, packed)]
pub struct FATBootSector {
    pub jmp_boot: [u8; 3],
    pub bs_oem_name: [u8; 8],
    pub bpb_bytes_per_sec: u16,
    pub bpb_sec_per_clus: u8,
    pub bpb_rsvd_sec_cnt: u16,
    pub bpb_num_fats: u8,
    pub bpb_root_ent_cnt: u16,
    pub bpb_tot_sec16: u16,
    pub bpb_media: u8,
    pub bpb_fat_size16: u16,
    pub bpb_sec_per_trk: u16,
    pub bpb_num_heads: u16,
    pub bpb_hidd_sec: u32,
    pub bpb_tot_sec32: u32,
    pub bpb_fat_size32: u32,
    pub bpb_ext_flags: u16,
    pub bpb_fs_ver: u16,
    pub bpb_root_clus: u32,
    pub fs_info: u16,
    pub bpb_bk_boot_sec: u16,
    pub bpb_reserved: [u8; 12],
    pub bs_drv_num: u8,
    pub bs_reserved1: u8,
    pub bs_boot_sig: u8,
    pub bs_vol_id: u32,
    pub bs_vol_lab: [u8; 11],
    pub bs_fil_sys_type: [u8; 8],
    pub padding: [u8; 420],
    pub signature_word: [u8; 2],
    pub more_padding: [u8; 2],
}

pub struct DskSizeToSecPerClus {
    pub disk_size: u32,
    pub sec_per_clus_val: u8,
}

pub const DSK_TABLE_FAT32: [DskSizeToSecPerClus; 6] = [
    DskSizeToSecPerClus {
        disk_size: 66600,
        sec_per_clus_val: 0,
    }, // disks up to 32.5 MB
    DskSizeToSecPerClus {
        disk_size: 532480,
        sec_per_clus_val: 1,
    }, // disks up to 260 MB
    DskSizeToSecPerClus {
        disk_size: 16777216,
        sec_per_clus_val: 8,
    }, // disks up to 8 GB
    DskSizeToSecPerClus {
        disk_size: 33554432,
        sec_per_clus_val: 16,
    }, // disks up to 16 GB
    DskSizeToSecPerClus {
        disk_size: 67108864,
        sec_per_clus_val: 32,
    }, // disks up to 32 GB
    DskSizeToSecPerClus {
        disk_size: 0xFFFFFFFF,
        sec_per_clus_val: 64,
    }, // disks greater than 32GB
];

pub fn get_disk_size() -> u32 {
    let mut sector_count: u16 = 0;

    // Using BIOS interrupt 0x13 to get disk size
    unsafe {
        asm!(
            "mov ah, 0x08",          // BIOS function 0x08: Get drive parameters
            "mov dl, 0x80",          // Select first hard disk (0x80)
            "int 0x13",                // BIOS interrupt 0x13
            inout("dx") sector_count => sector_count,
            options(nostack, preserves_flags)
        );
    }

    sector_count as u32
}

pub fn compute_fat_size(boot_sector: &mut FATBootSector) {
    let root_dir_sectors = ((boot_sector.bpb_root_ent_cnt as u32 * 32)
        + (boot_sector.bpb_bytes_per_sec as u32 - 1))
        / boot_sector.bpb_bytes_per_sec as u32;

    let tmp_val1 = get_disk_size() - (boot_sector.bpb_rsvd_sec_cnt as u32 + root_dir_sectors);

    let tmp_val2 = (256 * boot_sector.bpb_sec_per_clus as u32) + boot_sector.bpb_num_fats as u32;

    let tmp_val2 = tmp_val2 / 2;

    let fat_size = (tmp_val1 + (tmp_val2 - 1)) / tmp_val2;

    boot_sector.bpb_fat_size16 = 0;
    boot_sector.bpb_fat_size32 = fat_size;
}

pub fn check_if_fat32(boot_sector: &FATBootSector) -> bool {
    let root_dir_sectors = ((boot_sector.bpb_root_ent_cnt as u32 * 32)
        + (boot_sector.bpb_bytes_per_sec as u32 - 1))
        / boot_sector.bpb_bytes_per_sec as u32;
    let data_sec = boot_sector.bpb_tot_sec32
        - (boot_sector.bpb_rsvd_sec_cnt as u32
            + (boot_sector.bpb_num_fats as u32 * boot_sector.bpb_fat_size32)
            + root_dir_sectors);
    let cluster_count = data_sec / boot_sector.bpb_sec_per_clus as u32;

    cluster_count >= 65525
}

pub fn compute_fat_entry(boot_sector: &FATBootSector, cluster_number: u32) -> u32 {
    let fat_size = boot_sector.bpb_fat_size32;
    let fat_offset = cluster_number * 4;
    let mut sec_buff: Vec<u8> = vec![0; boot_sector.bpb_bytes_per_sec as usize];

    let sec_num =
        boot_sector.bpb_rsvd_sec_cnt as u32 + (fat_offset / boot_sector.bpb_bytes_per_sec as u32);
    let ent_offset = fat_offset % boot_sector.bpb_bytes_per_sec as u32;

    let clus_entry_val = u32::from_le_bytes(
        sec_buff[ent_offset as usize..ent_offset as usize + 4]
            .try_into()
            .unwrap(),
    ) & 0x0FFFFFFF;

    clus_entry_val
}

pub fn check_dirty_flags(boot_sector: &FATBootSector, cluster_number: u32) -> (bool, bool) {
    let fat_entry = compute_fat_entry(boot_sector, cluster_number);
    let clean_shutdown = (fat_entry & FAT32_CLEAN_SHUT_MASK) != 0;
    let hard_error = (fat_entry & FAT32_HARD_ERROR_MASK) != 0;
    return (clean_shutdown, hard_error);
}

/// returns in sectors.
pub fn determine_free_space(boot_sector: &FATBootSector) -> u32 {
    // Calculate the number of sectors in the root directory.
    let root_dir_sectors = ((boot_sector.bpb_root_ent_cnt as u32 * 32)
        + (boot_sector.bpb_bytes_per_sec as u32 - 1))
        / boot_sector.bpb_bytes_per_sec as u32;

    // Calculate the number of sectors used by all FATs.
    let fat_sectors = boot_sector.bpb_num_fats as u32 * boot_sector.bpb_fat_size32;

    // Calculate the total data sectors.
    let total_data_sectors = boot_sector.bpb_tot_sec32
        - (boot_sector.bpb_rsvd_sec_cnt as u32 + fat_sectors + root_dir_sectors);

    // Calculate the maximum number of clusters.
    let total_data_clusters = total_data_sectors / boot_sector.bpb_sec_per_clus as u32;

    let mut free_secs: u32 = 0;
    for cluster in 2..=total_data_clusters {
        let cluster_entry = compute_fat_entry(boot_sector, cluster);
        if cluster_entry == 0x0000000 {
            free_secs += boot_sector.bpb_sec_per_clus as u32;
        }
    }

    return free_secs;
}

#[repr(C, packed)]
pub struct FSInfo {
    pub fsi_lead_sig: u32,
    pub fsi_reserved1: [u8; 480],
    pub fsi_struc_sig: u32,
    pub fsi_free_count: u32,
    pub fsi_nxt_free: u32,
    pub fsi_reserved2: [u8; 12],
    pub fsi_trail_sig: u32,
}

enum Attributes {
    AttrReadOnly = 0x01,
    AttrHidden = 0x02,
    AttrSystem = 0x04,
    AttrVolumeID = 0x08,
    AttrDirectory = 0x10,
    AttrArchive = 0x20,
}

/// Each directory MUST have a "." and ".." directory inside.
#[repr(C, packed)]
pub struct Directory {
    /// 8 character name & 3 character extension.
    /// the "." in the extension is not stored, rather implied.
    /// illegal values are as follows:
    /// 0x22, 0x2A, 0x2B, 0x2C, 0x2E, 0x2F, 0x3A,
    /// 0x3B, 0x3C, 0x3D, 0x3E, 0x3F, 0x5B, 0x5C,
    /// 0x5D, 0x7C.
    pub dir_name: [u8; 11],
    /// Must be AttrDirectory (0x10), unless the
    /// directory is the root directory, then it
    /// would be AttrVolumeID (0x08).
    pub dir_attr: u8,
    /// reserved. MUST be zero.
    pub dir_ntres: u8,
    pub dir_crl_time_tenth: u8,
    pub dir_crl_time: u16,
    pub dir_crl_date: u16,
    pub dir_lst_acc_date: u16,
    /// Must refer to first allocated cluster number
    pub dir_fst_clus_high: u16,
    /// this time is NOT optional!
    pub dir_wrt_time: u16,
    /// this time is NOT optional!
    pub dir_wrt_date: u16,
    /// Must refer to first allocated cluster number
    pub dir_fst_clus_low: u16,
    /// Must be zero on creation.
    pub dir_file_size: u32,
}

pub struct LongFileNames {
    pub ldir_ord: u8,
    pub ldir_name1: [u8; 10],
    pub ldir_attr: u8,
    pub ldir_type: u8,
    pub ldir_chksum: u8,
    pub ldir_name2: [u8; 12],
    pub ldir_fst_clus_low: u16,
    pub ldir_name3: u32,
}

pub fn long_file_name_checksum(p_fcb_name: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    let mut fcb_name_len: i16 = 11; // Using i16 to match the short type in C

    let mut p_fcb_name_iter = p_fcb_name.iter();

    while fcb_name_len != 0 {
        fcb_name_len -= 1;

        // Perform the unsigned char rotate right operation
        sum = if sum & 1 != 0 { 0x80 } else { 0 }
            + (sum >> 1)
            + *p_fcb_name_iter.next().unwrap_or(&0);
    }

    sum
}

pub fn create_file(
    directory: &mut Directory,
    name: &str,
    extension: &str,
    attr: u8,
    cluster: u32,
) -> Result<(), &'static str> {
    if name.len() > 8 || extension.len() > 3 {
        return Err("Name or extension too long");
    }

    let mut dir_name = [b' '; 11];
    for (i, byte) in name.bytes().enumerate() {
        dir_name[i] = byte;
    }
    for (i, byte) in extension.bytes().enumerate() {
        dir_name[8 + i] = byte;
    }

    directory.dir_name = dir_name;
    directory.dir_attr = attr;
    directory.dir_fst_clus_high = (cluster >> 16) as u16;
    directory.dir_fst_clus_low = cluster as u16;
    directory.dir_file_size = 0;

    Ok(())
}

pub fn find_free_cluster(fat: &mut [u32]) -> Option<u32> {
    for (i, &entry) in fat.iter().enumerate() {
        if entry == 0x00000000 {
            return Some(i as u32);
        }
    }
    None
}

pub fn update_fat(fat: &mut [u32], cluster: u32, next_cluster: u32) {
    fat[cluster as usize] = next_cluster & 0x0FFFFFFF;
}

pub fn write_file(
    fat: &mut [u32],
    data_region: &mut [u8],
    boot_sector: &FATBootSector,
    start_cluster: u32,
    data: &[u8],
) -> Result<(), &'static str> {
    let mut current_cluster = start_cluster;
    let mut remaining_data = data;

    while !remaining_data.is_empty() {
        let cluster_size =
            boot_sector.bpb_sec_per_clus as usize * boot_sector.bpb_bytes_per_sec as usize;
        let offset = ((current_cluster - 2)
            * boot_sector.bpb_sec_per_clus as u32
            * boot_sector.bpb_bytes_per_sec as u32) as usize;

        let to_write = remaining_data.len().min(cluster_size);
        data_region[offset..offset + to_write].copy_from_slice(&remaining_data[..to_write]);
        remaining_data = &remaining_data[to_write..];

        if !remaining_data.is_empty() {
            match find_free_cluster(fat) {
                Some(next_cluster) => {
                    update_fat(fat, current_cluster, next_cluster);
                    current_cluster = next_cluster;
                }
                None => return Err("No free clusters available"),
            }
        } else {
            update_fat(fat, current_cluster, 0x0FFFFFFF); // Mark as end of chain
        }
    }

    Ok(())
}
