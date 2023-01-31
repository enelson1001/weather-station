use std::thread;
use std::time::Duration;
use std::time::Instant;

use crossbeam_channel::Sender;
use time::{format_description, OffsetDateTime, UtcOffset};

const UTC_OFFSET_STR: &str = env!("UTC_OFFSET");
const DATE_STR: &'static str = "[weekday repr:short], [day] [month repr:short] [year]";
const TIME_STR: &'static str = "[hour repr:12 padding:none]:[minute] [period case:upper]";

pub enum TimeDate {
    Time(String),
    Date(String),
}

pub enum TimeEvent {
    TwoMinutesElapsed,
    OneHourElapsed,
    NewDay,
    NewMonth,
    NewYear,
}
struct Senders {
    tx1: Sender<TimeDate>,
    tx2: Sender<TimeEvent>,
}

struct PreviousTime {
    minute_timer: Instant,
    hour_timer: Instant,
    minute: u8,
    day: u8,
    month: u8,
    year: i32,
}

impl Default for PreviousTime {
    fn default() -> Self {
        Self {
            minute_timer: Instant::now(),
            hour_timer: Instant::now(),
            minute: 0,
            day: 0,
            month: 0,
            year: 2019,
        }
    }
}

pub struct Scheduler {
    sender: Senders,
    previous_time: PreviousTime,
}

impl Scheduler {
    pub fn new(tx1: Sender<TimeDate>, tx2: Sender<TimeEvent>) -> Self {
        Self {
            sender: Senders { tx1, tx2 },
            previous_time: Default::default(),
        }
    }

    pub fn start(mut self) {
        println!("Starting Scheduler Thread");

        let _scheduler_thread = std::thread::Builder::new()
            .stack_size(4096)
            .spawn(move || loop {
                let utc_offset: i8 = UTC_OFFSET_STR.parse().unwrap();
                let mountain_time_zone = UtcOffset::from_hms(utc_offset, 0, 0).unwrap();

                loop {
                    if self.previous_time.minute_timer.elapsed().as_millis() > 1000 * 60 * 2 {
                        self.sender.tx2.send(TimeEvent::TwoMinutesElapsed).unwrap();
                        self.previous_time.minute_timer = Instant::now();
                    }

                    if self.previous_time.hour_timer.elapsed().as_secs() > 60 * 60 {
                        self.sender.tx2.send(TimeEvent::OneHourElapsed).unwrap();
                        self.previous_time.hour_timer = Instant::now();
                    }

                    let dt = OffsetDateTime::now_utc().to_offset(mountain_time_zone);

                    let minute = dt.minute();
                    if minute != self.previous_time.minute {
                        let time_fmt = format_description::parse(TIME_STR).unwrap();
                        let time_str = dt.format(&time_fmt).unwrap();
                        self.sender.tx1.send(TimeDate::Time(time_str)).unwrap();
                        self.previous_time.minute = minute;
                    }

                    let day = dt.day();
                    if day != self.previous_time.day {
                        let date_fmt = format_description::parse(DATE_STR).unwrap();
                        let date_str = dt.format(&date_fmt).unwrap();
                        self.sender.tx1.send(TimeDate::Date(date_str)).unwrap();
                        self.sender.tx2.send(TimeEvent::NewDay).unwrap();
                        self.previous_time.day = day;
                    }

                    let month: u8 = dt.month().into();
                    if month != self.previous_time.month {
                        self.sender.tx2.send(TimeEvent::NewMonth).unwrap();
                        self.previous_time.month = month;
                    }

                    let year = dt.year();
                    if year != self.previous_time.year {
                        self.sender.tx2.send(TimeEvent::NewYear).unwrap();
                        self.previous_time.year = year;
                    }

                    thread::sleep(Duration::from_secs(1));
                }
            });
    }
}
