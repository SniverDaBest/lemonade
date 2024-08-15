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

    pub fn gen_range() {

    }
}

pub fn gen_number(seed: u32) -> u32 { return Xorshift32::new(seed).next(); }
