use alloc::string::{String, ToString};
use x86_64::instructions::random::*;

pub fn rand_u16() -> Result<Option<u16>, String> {
    let rnd = RdRand::new();
    if rnd.is_some() {
        Ok(RdRand::get_u16(rnd.unwrap()))
    } else {
        Err("Failed to generate random value.".to_string())
    }
}

pub fn rand_u32() -> Result<Option<u32>, String> {
    let rnd = RdRand::new();
    if rnd.is_some() {
        Ok(RdRand::get_u32(rnd.unwrap()))
    } else {
        Err("Failed to generate random value.".to_string())
    }
}

pub fn rand_u64() -> Result<Option<u64>, String> {
    let rnd = RdRand::new();
    if rnd.is_some() {
        Ok(RdRand::get_u64(rnd.unwrap()))
    } else {
        Err("Failed to generate random value.".to_string())
    }
}