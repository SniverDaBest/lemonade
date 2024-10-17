use bootloader::BootInfo;

pub struct Framebuffer {
    address: *mut u8,     // Pointer to the framebuffer memory
    width: usize,         // Screen width in pixels
    height: usize,        // Screen height in pixels
    pitch: usize,         // Number of bytes per row (may be greater than width * bytes_per_pixel)
    bytes_per_pixel: usize,
}

impl Framebuffer {
    pub fn new(address: *mut u8, width: usize, height: usize, pitch: usize, bytes_per_pixel: usize) -> Self {
        return Self {
            address,
            width,
            height,
            pitch,
            bytes_per_pixel,
        }
    }

    pub fn put_pixel(&self, x: usize, y: usize, color: u32) {
        // Ensure coordinates are within screen bounds
        if x >= self.width || y >= self.height {
            return;
        }

        let pixel_index = (y * self.pitch + x * self.bytes_per_pixel) as isize;

        // Assuming 32-bit RGBA format
        unsafe {
            let pixel_ptr = self.address.offset(pixel_index) as *mut u32;
            *pixel_ptr = color;
        }
    }

    pub fn fill_rect(&self, x: usize, y: usize, xx: usize, yy: usize, color: u32) {
        for xpos in x..xx {
            for ypos in y..yy {
                self.put_pixel(xpos, ypos, color);
            }
        }
    }

    pub fn empty_rect(&self, x: usize, y: usize, xx: usize, yy: usize, color: u32) {
        for xpos in x..xx {
            self.put_pixel(xpos, y, color);
        }
        for xpos in x..xx {
            self.put_pixel(xpos, yy, color);
        }
        for ypos in y..yy {
            self.put_pixel(x, ypos, color);
        }
        for ypos in y..yy {
            self.put_pixel(xx, ypos, color);
        }
    }
}
