use ds323x::{ic::DS3231, interface::I2cInterface, DateTimeAccess, Ds323x, NaiveDate, NaiveDateTime};
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_sys::{settimeofday, timeval, timezone};
use shared_bus::I2cProxy;
use std::sync::Mutex;
use time::OffsetDateTime;

pub struct RealTimeClock<'a> {
    rtc: Ds323x<I2cInterface<I2cProxy<'a, Mutex<I2cDriver<'static>>>>, DS3231>,
}

#[allow(dead_code)]
impl<'a> RealTimeClock<'a> {
    pub fn new(rtc: Ds323x<I2cInterface<I2cProxy<'a, Mutex<I2cDriver<'static>>>>, DS3231>) -> Self {
        Self { rtc: (rtc) }
    }

    pub fn set_system_clock(&mut self) {
        

        let dt = self.rtc.datetime().unwrap();
        let tz = timezone {
            tz_minuteswest: 0,
            tz_dsttime: 0,
        };

        let tv_sec = dt.timestamp() as i32;
        let tv_usec = dt.timestamp_subsec_micros() as i32;
        let tm = timeval { tv_sec, tv_usec };

        unsafe {
            settimeofday(&tm, &tz);
        }

        println!(
            "Updated System Clock from RTC --- time now is {} ",
            OffsetDateTime::now_utc()
        );
    }

    pub fn set_date_time(&mut self, year: i32, month: u32, day: u32, hour: u32, minute: u32, second: u32) {
        let datetime = NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(hour, minute, second)
            .unwrap();

        self.rtc.set_datetime(&datetime).unwrap();
    }

    pub fn set_date_time_from_naive_date_time(&mut self, date_time:NaiveDateTime) {
        self.rtc.set_datetime(&date_time).unwrap();
    }
}
