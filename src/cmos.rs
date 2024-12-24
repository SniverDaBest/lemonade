use core::ffi::*;

extern "C" {
    pub static second: c_uchar;
    pub static minute: c_uchar;
    pub static hour: c_uchar;
    pub static day: c_uchar;
    pub static month: c_uchar;
    pub static year: c_int;
    pub static cmos_address: c_int;
    pub static cmos_data: c_int;
    pub fn read_rtc();
}

pub struct Time {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: i32,
}

impl Time {
    pub fn from_current() -> Self {
        unsafe {
            read_rtc();
            let sec: u8 = second.into();
            let min: u8 = minute.into();
            let hr: u8 = hour.into();
            let d: u8 = day.into();
            let m: u8 = month.into();
            let yr: i32 = year.into();
            return Self {
                second: sec,
                minute: min,
                hour: hr,
                day: d,
                month: m,
                year: yr,
            };
        }
    }

    pub fn update(&mut self) {
        unsafe {
            read_rtc();
            self.second = second;
            self.minute = minute;
            self.hour = hour;
            self.day = day;
            self.month = month;
            self.year = year;
        }
    }
}

impl core::fmt::Display for Time {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{}/{}/{} {}:{}:{}",
            self.month, self.day, self.year, self.hour, self.minute, self.second
        )
    }
}
