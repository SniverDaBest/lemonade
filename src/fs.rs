#![allow(arithmetic_overflow)]

use alloc::{
    string::*,
    vec,
    vec::*,
};

/// The header of the filesystem.
pub struct FSHeader {
    /// Filesystem version.
    pub version: u8,
    /// Name of partion filesystem is on.
    pub part_name: String,
}

impl FSHeader {
    pub fn new(version: u8, part_name: String) -> Self { return Self {version, part_name}; }
}

/// A node of the filesystem.
pub struct FSNode {
    /// File path & name.
    pub name: String,
    /// 0 for file, 1 for directory.
    pub node_type: u8,
    /// The file length.
    pub file_len: u128,
    /// 0 for last entry in FS, otherwise 1.
    pub next_byte: u8,
}

impl FSNode {
    pub fn new(name: String, node_type: u8, file_len: u128, next_byte: u8) -> Self { return Self {name, node_type, file_len, next_byte}; }
}

/// Writes to a filesystem header.
pub fn write_fs_header(result: &mut Vec<u8>, header: &mut FSHeader) {
    result[0] = header.version >> 24 & 0xff;
    result[1] = header.version >> 16 & 0xff;
    result[2] = header.version >> 8 & 0xff;
    result[3] = header.version & 0xff;
    for i in 0..32 { result[4+i] = header.part_name.as_bytes()[i]; }
}

/// Writes to a filesystem node.
pub fn write_fs_node(result: &mut Vec<u8>, node: &mut FSNode) {
    for i in 0..255 { result[i] = node.name.as_bytes()[i] }
    result[256] = node.node_type;
    result[257] = (node.file_len >> 56) as u8;
    result[258] = (node.file_len >> 48) as u8;
    result[259] = (node.file_len >> 40) as u8;
    result[260] = (node.file_len >> 32) as u8;
    result[261] = (node.file_len >> 24) as u8;
    result[262] = (node.file_len >> 16) as u8;
    result[263] = (node.file_len >> 8) as u8;
    result[264] = node.file_len as u8;
    result[265] = node.next_byte;
}

/// Serializes an FSHeader into a byte vec.
pub fn serialize_fs_header(header: &FSHeader) -> Vec<u8> {
    let mut result: Vec<u8> = vec![0; 36];
    result[0] = header.version;
    let name_bytes = header.part_name.as_bytes();
    result[4..4 + name_bytes.len()].copy_from_slice(name_bytes);
    return result;
}

/// Serializes an FSNode into a byte vec.
pub fn serialize_fs_node(node: &FSNode) -> Vec<u8> {
    let mut result = vec![0; 266];
    let name_bytes = node.name.as_bytes();
    result[0..name_bytes.len()].copy_from_slice(name_bytes);
    result[256] = node.node_type;
    result[257..265].copy_from_slice(&node.file_len.to_be_bytes());
    result[265] = node.next_byte;
    return result;
}

/// Creates a directory node.
pub fn create_dir_node(path: String) -> FSNode {
    return FSNode::new(path, 1, 0, 1);
}

/// Writes to a new file.
pub fn write_to_file(path: String, data: &[u8]) -> FSNode {
    let file_len = data.len() as u128;
    return FSNode::new(path, 0, file_len, 1);
}