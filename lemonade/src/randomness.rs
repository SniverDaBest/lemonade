use alloc::{
    format,
    string::{String, ToString},
};
use x86_64::instructions::random::*;

pub fn rand_u16() -> Result<Option<u16>, String> {
    let rnd = RdRand::new();
    if rnd.is_some() {
        Ok(RdRand::get_u16(rnd.unwrap()))
    } else {
        Err("(-_-)  Failed to generate random value.".to_string())
    }
}

pub fn rand_u32() -> Result<Option<u32>, String> {
    let rnd = RdRand::new();
    if rnd.is_some() {
        Ok(RdRand::get_u32(rnd.unwrap()))
    } else {
        Err("(-_-)  Failed to generate random value.".to_string())
    }
}

pub fn rand_u64() -> Result<Option<u64>, String> {
    let rnd = RdRand::new();
    if rnd.is_some() {
        Ok(RdRand::get_u64(rnd.unwrap()))
    } else {
        Err("(-_-)  Failed to generate random value.".to_string())
    }
}

pub fn gen_range_u16(min: u16, max: u16) -> Result<Option<u16>, String> {
    let mut res = rand_u16();
    match res {
        Ok(r) => {
            while r.unwrap() < min && r.unwrap() > max {
                res = rand_u16();
            }
            return res;
        }
        Err(e) => {
            return Err(
                format!("(-_-)  Unable to generate random range. Details: {}", e).to_string(),
            )
        }
    }
}

pub struct Xorshift32 {
    state: u32,
}

impl Xorshift32 {
    pub fn new(seed: u32) -> Self {
        Xorshift32 { state: seed }
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    pub fn gen_range(&mut self, min: u32, max: u32) -> u32 {
        let mut res = self.next();
        while res >= min && res <= max {
            res = self.next();
        }
        return res;
    }
}
