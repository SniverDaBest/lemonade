#![no_std]

const CURRENT_YEAR: u16 = 2024;

static mut CENTURY_REGISTER: u8 = 0x00;

static mut SECOND: u8 = 0;
static mut MINUTE: u8 = 0;
static mut HOUR: u8 = 0;
static mut DAY: u8 = 0;
static mut MONTH: u8 = 0;
static mut YEAR: u16 = 0;

fn out_byte(port: u32, value: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nostack, preserves_flags)
        );
    }
}

fn in_byte(port: u32) -> u8 {
    let mut ret: u8 = 0;
    unsafe {
        core::arch::asm!(
            "in al, dx",
            out("al") ret,
            in("dx") port as u16,
            options(nostack, preserves_flags)
        );
    }
    ret
}

const CMOS_ADDRESS: u32 = 0x70;
const CMOS_DATA: u32 = 0x71;

fn get_update_in_progress_flag() -> bool {
    out_byte(CMOS_ADDRESS, 0x0A);
    (in_byte(CMOS_DATA) & 0x80) != 0
}

fn get_RTC_register(reg: u8) -> u8 {
    out_byte(CMOS_ADDRESS, reg);
    in_byte(CMOS_DATA) as u8
}

fn read_rtc() {
    let mut century: u8 = 0;
    let mut last_second: u8 = 0;
    let mut last_minute: u8 = 0;
    let mut last_hour: u8 = 0;
    let mut last_day: u8 = 0;
    let mut last_month: u8 = 0;
    let mut last_year: u16 = 0;
    let mut last_century: u8 = 0;
    let mut register_b: u8 = 0;

    // Note: This uses the "read registers until you get the same values twice in a row" technique
    //       to avoid getting dodgy/inconsistent values due to RTC updates

    while get_update_in_progress_flag() {}
    unsafe {
        SECOND = get_RTC_register(0x00);
        MINUTE = get_RTC_register(0x02);
        HOUR = get_RTC_register(0x04);
        DAY = get_RTC_register(0x07);
        MONTH = get_RTC_register(0x08);
        YEAR = get_RTC_register(0x09) as u16;
        if CENTURY_REGISTER != 0 {
            century = get_RTC_register(CENTURY_REGISTER);
        }
    }

    loop {
        last_second = unsafe { SECOND };
        last_minute = unsafe { MINUTE };
        last_hour = unsafe { HOUR };
        last_day = unsafe { DAY };
        last_month = unsafe { MONTH };
        last_year = unsafe { YEAR };
        last_century = century;

        while get_update_in_progress_flag() {
            unsafe {
                SECOND = get_RTC_register(0x00);
                MINUTE = get_RTC_register(0x02);
                HOUR = get_RTC_register(0x04);
                DAY = get_RTC_register(0x07);
                MONTH = get_RTC_register(0x08);
                YEAR = get_RTC_register(0x09) as u16;
                if CENTURY_REGISTER != 0 {
                    century = get_RTC_register(CENTURY_REGISTER);
                }
            }

            if last_second == unsafe { SECOND }
                && last_minute == unsafe { MINUTE }
                && last_hour == unsafe { HOUR }
                && last_day == unsafe { DAY }
                && last_month == unsafe { MONTH }
                && last_year == unsafe { YEAR }
                && last_century == century
            {
                break;
            }
        }

        register_b = get_RTC_register(0x0B);

        // Convert BCD to binary values if necessary

        if (register_b & 0x04) == 0 {
            unsafe {
                SECOND = (SECOND & 0x0F) + ((SECOND / 16) * 10);
                MINUTE = (MINUTE & 0x0F) + ((MINUTE / 16) * 10);
                HOUR = ((HOUR & 0x0F) + (((HOUR & 0x70) / 16) * 10)) | (HOUR & 0x80);
                DAY = (DAY & 0x0F) + ((DAY / 16) * 10);
                MONTH = (MONTH & 0x0F) + ((MONTH / 16) * 10);
                YEAR = (YEAR & 0x0F) + ((YEAR / 16) * 10);
                if CENTURY_REGISTER != 0 {
                    century = (century & 0x0F) + ((century / 16) * 10);
                }
            }
        }

        // Convert 12 hour clock to 24 hour clock if necessary

        if (register_b & 0x02) == 0 && (unsafe { HOUR } & 0x80) != 0 {
            unsafe {
                HOUR = ((HOUR & 0x7F) + 12) % 24;
            }
        }

        // Calculate the full (4-digit) year
        unsafe {
            if CENTURY_REGISTER != 0 {
                YEAR += century as u16 * 100;
            } else {
                YEAR += (CURRENT_YEAR / 100) * 100;
                if YEAR < CURRENT_YEAR {
                    YEAR += 100;
                }
            }
        }
    }
}

pub struct Time {
    pub sec: u8,
    pub min: u8,
    pub hr: u8,
    pub day: u8,
    pub mon: u8,
    pub yr: u16,
}

impl Time {
    pub fn from_current() -> Self {
        read_rtc();
        unsafe {
            Time { sec: SECOND, min: MINUTE, hr: HOUR, day: DAY, mon: MONTH, yr: YEAR }
        }
    }
}

impl core::fmt::Display for Time {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}/{}/{} {}:{}:{}", self.mon, self.day, self.yr, self.hr, self.min, self.sec)
    }
}