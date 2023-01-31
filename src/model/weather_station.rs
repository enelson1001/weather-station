use crate::model::{
    acurite5n1::{Acurite5n1Message, MessageHeader, MessageType1, MessageType8},
    scheduler::TimeEvent,
};

use crossbeam_channel::{Receiver, Sender};
use esp_idf_hal::delay;
use esp_idf_hal::i2c::I2cDriver;
use shared_bus::I2cProxy;

use bme280_rs::{Bme280, Configuration, Oversampling, SensorMode};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

const MY_ALTITUDE_METERS: &str = env!("MY_ALTITUDE_METERS");

pub enum Measurement {
    BatteryStatus(String),
    ChannelNumber(u8),
    ProductId(u16),
    CurrentWindSpeedMph(u8),
    AverageWindSpeedMph(u8),
    PeakWindSpeedMph(u8),
    WindDirection(String),
    DailyRainfall(f32),
    MonthlyRainfall(f32),
    YearlyRainfall(f32),
    OutdoorTemperature(i16),
    OutdoorHumidity(u8),
    OutdoorHeatIndex(i16),
    OutdoorWindchill(i16),
    OutdoorDewpoint(i16),
    IndoorTemperature(i16),
    IndoorHumidity(u8),
    IndoorPressure(f32),
}

#[derive(Default)]
pub struct LastRainfall {
    pub daily: f32,
    pub monthly: f32,
    pub yearly: f32,
}

#[derive(Default)]
pub struct LastIndoorSample {
    pub temperaturex10: i16,
    pub humidityx10: u16,
    pub pressurex10: u32,
}

pub struct HeatIndex {
    pub fahrenheit: i16,
    pub celsius: i16,
}

pub struct Windchill {
    pub fahrenheit: i16,
    pub celsius: i16,
}

pub struct Dewpoint {
    pub fahrenheit: i16,
    pub celsius: i16,
}

pub struct OutdoorTemperature {
    pub fahrenheit: i16,
    pub celsius: i16,
}

/*
pub struct IndoorTemperature {
    pub fahrenheit: i16,
    pub celsius: i16,
}
*/

pub struct LastRawMeasurement {
    channel: u8,
    prodcut_id: u16,
    status: u8,
    wind_speed: u8,
    wind_direction: u8,
    rain_bucket_tips: u16,
    humidity: u8,
    temperature: u16,
}

impl Default for LastRawMeasurement {
    fn default() -> Self {
        Self {
            channel: u8::MAX,
            prodcut_id: 0,
            status: u8::MAX,
            wind_speed: u8::MAX,
            wind_direction: 0,
            rain_bucket_tips: u16::MAX,
            humidity: u8::MAX,
            temperature: u16::MAX,
        }
    }
}

pub struct WeatherStation {
    //i2c_proxy: I2cProxy<'a, Mutex<I2cDriver<'static>>>,
    bme280: Bme280<I2cProxy<'static, Mutex<I2cDriver<'static>>>, delay::Ets>,
    rx1: Receiver<Acurite5n1Message>, // Receive from Acurite5n1
    rx2: Receiver<TimeEvent>,         // Receive from Scheduler
    tx1: Sender<Measurement>,
    last_raw_measurement: LastRawMeasurement,
    last_peak_wind_speed_mph: u8,
    last_rainfall: LastRainfall,
    collected_wind_speeds_mph: Vec<u8>,
    last_indoor_sample: LastIndoorSample,
}

impl WeatherStation {
    pub fn new(
        i2c_proxy: I2cProxy<'static, Mutex<I2cDriver<'static>>>,
        rx1: Receiver<Acurite5n1Message>,
        rx2: Receiver<TimeEvent>,
        tx1: Sender<Measurement>,
    ) -> Self {
        Self {
            //i2c_proxy,
            bme280: Bme280::new(i2c_proxy, delay::Ets),
            rx1,
            rx2,
            tx1,
            last_raw_measurement: LastRawMeasurement::default(),
            last_peak_wind_speed_mph: 0,
            last_rainfall: LastRainfall::default(),
            collected_wind_speeds_mph: Vec::new(),
            last_indoor_sample: LastIndoorSample::default(),
        }
    }

    pub fn start(mut self) {
        println!("Starting WeatherStation Thread");

        self.bme280.init().expect("Bme280 not connected?????");
        self.bme280
            .set_sampling_configuration(
                Configuration::default()
                    .with_temperature_oversampling(Oversampling::Oversample1)
                    .with_pressure_oversampling(Oversampling::Oversample8)
                    .with_humidity_oversampling(Oversampling::Oversample1)
                    .with_sensor_mode(SensorMode::Normal),
            )
            .expect("Failed to configure bme280");

        // my_elevation value was determined from using "My Elevation" app on android phone at my location.
        let my_elevation: f32 = MY_ALTITUDE_METERS.parse().unwrap();

        let _weather_station_thread =
            std::thread::Builder::new()
                .stack_size(5000)
                .spawn(move || loop {
                    while let Ok(time_event) = self.rx2.try_recv() {
                        match time_event {
                            TimeEvent::TwoMinutesElapsed => {
                                self.update_average_wind_speed();
                                self.process_bme280(my_elevation);
                            }

                            TimeEvent::OneHourElapsed => self.last_peak_wind_speed_mph = 0,

                            TimeEvent::NewDay => {
                                self.last_rainfall.daily = 0.0;
                                self.tx1
                                    .send(Measurement::DailyRainfall(0.0 as f32))
                                    .unwrap();
                            }

                            TimeEvent::NewMonth => {
                                self.last_rainfall.monthly = 0.0;
                                self.tx1
                                    .send(Measurement::MonthlyRainfall(0.0 as f32))
                                    .unwrap();
                            }

                            TimeEvent::NewYear => {
                                self.last_rainfall.yearly = 0.0;
                                self.tx1
                                    .send(Measurement::YearlyRainfall(0.0 as f32))
                                    .unwrap();
                            }
                        }
                    }

                    if let Ok(message) = self.rx1.try_recv() {
                        self.process_message(message);
                    }

                    thread::sleep(Duration::from_secs(1));
                });
    }

    fn process_message(&mut self, message: Acurite5n1Message) {
        match message {
            Acurite5n1Message::Type1(MessageType1 {
                header,
                wind_speed,
                wind_direction,
                rain_bucket_tips,
            }) => {
                self.process_header(header);
                self.process_wind_speed(wind_speed);
                self.process_wind_direction(wind_direction);
                self.process_rain_bucket_tips(rain_bucket_tips);
            }

            Acurite5n1Message::Type8(MessageType8 {
                header,
                wind_speed,
                humidity,
                temperature,
            }) => {
                self.process_header(header);
                self.process_wind_speed(wind_speed);
                self.process_temperature_humidity_wind_speed(temperature, humidity, wind_speed);
            }
        }
    }

    fn process_header(&mut self, header: MessageHeader) {
        if header.channel_number != self.last_raw_measurement.channel {
            self.last_raw_measurement.channel = header.channel_number;
            self.tx1
                .send(Measurement::ChannelNumber(header.channel_number))
                .unwrap();
        }

        if header.product_id != self.last_raw_measurement.prodcut_id {
            self.last_raw_measurement.prodcut_id = header.product_id;
            self.tx1
                .send(Measurement::ProductId(header.product_id))
                .unwrap();
        }

        if header.status != self.last_raw_measurement.status {
            self.last_raw_measurement.status = header.status;

            let battery_status: &str;

            if header.status == 7 {
                battery_status = "OK";
            } else {
                battery_status = "REPLACE";
            }

            self.tx1
                .send(Measurement::BatteryStatus(battery_status.to_string()))
                .unwrap();
        }
    }

    fn process_wind_speed(&mut self, wind_speed: u8) {
        if wind_speed != self.last_raw_measurement.wind_speed {
            self.last_raw_measurement.wind_speed = wind_speed;

            let current_wind_speed_mph = self.convert_raw_wind_speed(wind_speed);

            // Collect wind speed mph for average wind speed calculation
            self.collected_wind_speeds_mph.push(current_wind_speed_mph);

            // Check if peak wind speed need to be updated
            if current_wind_speed_mph > self.last_peak_wind_speed_mph {
                self.last_peak_wind_speed_mph = current_wind_speed_mph;
                self.tx1
                    .send(Measurement::PeakWindSpeedMph(current_wind_speed_mph))
                    .unwrap();
            }

            self.tx1
                .send(Measurement::CurrentWindSpeedMph(current_wind_speed_mph))
                .unwrap();
        }
    }

    fn process_wind_direction(&mut self, wind_direction: u8) {
        if self.last_raw_measurement.wind_direction != wind_direction {
            self.last_raw_measurement.wind_direction = wind_direction;
            self.tx1
                .send(Measurement::WindDirection(
                    self.convert_raw_wind_direction(wind_direction),
                ))
                .unwrap();
        }
    }

    fn process_rain_bucket_tips(&mut self, rain_bucket_tips: u16) {
        // Check if this is the first time processing rain bucket tips signified by u16::MAX
        // The rain gauge data is a counter of bucket tips that ranges from 0 to 16383 (14 bits)
        if self.last_raw_measurement.rain_bucket_tips == u16::MAX {
            self.last_raw_measurement.rain_bucket_tips = rain_bucket_tips;
        }

        if self.last_raw_measurement.rain_bucket_tips != rain_bucket_tips {
            let rainfall = self.convert_raw_rain_bucket_tips(
                rain_bucket_tips - self.last_raw_measurement.rain_bucket_tips,
            );

            self.last_raw_measurement.rain_bucket_tips = rain_bucket_tips;

            self.last_rainfall.daily += rainfall;
            self.tx1
                .send(Measurement::DailyRainfall(self.last_rainfall.daily))
                .unwrap();

            self.last_rainfall.monthly += rainfall;
            self.tx1
                .send(Measurement::MonthlyRainfall(self.last_rainfall.monthly))
                .unwrap();

            self.last_rainfall.yearly += rainfall;
            self.tx1
                .send(Measurement::YearlyRainfall(self.last_rainfall.yearly))
                .unwrap();
        }
    }

    fn process_temperature_humidity_wind_speed(
        &mut self,
        raw_temperature: u16,
        current_humidity: u8,
        raw_wind_speed: u8,
    ) {
        let current_wind_speed_mph = self.convert_raw_wind_speed(raw_wind_speed);
        let current_temperature_deg_f = self.convert_raw_temperature(raw_temperature);
        let mut humidity_changed = false;
        let mut temperature_changed = false;
        let mut wind_speed_changed = false;

        // Check if temperature changed
        if self.last_raw_measurement.temperature != raw_temperature {
            temperature_changed = true;
            self.last_raw_measurement.temperature = raw_temperature;

            self.tx1
                .send(Measurement::OutdoorTemperature(
                    self.calculate_outdoor_temperature(raw_temperature)
                        .fahrenheit,
                ))
                .unwrap();
        }

        // Check if humidity changed
        if self.last_raw_measurement.humidity != current_humidity {
            humidity_changed = true;
            self.last_raw_measurement.humidity = current_humidity;

            self.tx1
                .send(Measurement::OutdoorHumidity(current_humidity))
                .unwrap();
        }

        // Check if wind speed changed
        if raw_wind_speed != self.last_raw_measurement.wind_speed {
            wind_speed_changed = true;
            self.last_raw_measurement.wind_speed = raw_wind_speed;

            // Collect wind speed mph for average wind speed calculation
            self.collected_wind_speeds_mph.push(current_wind_speed_mph);

            if current_wind_speed_mph > self.last_peak_wind_speed_mph {
                self.last_peak_wind_speed_mph = current_wind_speed_mph;
                self.tx1
                    .send(Measurement::PeakWindSpeedMph(current_wind_speed_mph))
                    .unwrap();
            }

            self.tx1
                .send(Measurement::CurrentWindSpeedMph(current_wind_speed_mph))
                .unwrap();
        }

        // Check if heat index and dew point need to be updated
        if temperature_changed || humidity_changed {
            self.tx1
                .send(Measurement::OutdoorHeatIndex(
                    self.calculate_heat_index(current_temperature_deg_f, current_humidity)
                        .fahrenheit,
                ))
                .unwrap();

            self.tx1
                .send(Measurement::OutdoorDewpoint(
                    self.calculate_dew_point(current_temperature_deg_f, current_humidity)
                        .fahrenheit,
                ))
                .unwrap();
        }

        // Check if wind chill needs to be updated
        if wind_speed_changed || temperature_changed {
            if current_wind_speed_mph > 3 && current_temperature_deg_f < 40.0 {
                self.tx1
                    .send(Measurement::OutdoorWindchill(
                        self.calculate_wind_chill(
                            current_wind_speed_mph,
                            current_temperature_deg_f,
                        )
                        .fahrenheit,
                    ))
                    .unwrap();
            } else {
                self.tx1
                    .send(Measurement::OutdoorWindchill(
                        current_temperature_deg_f as i16,
                    ))
                    .unwrap();
            }
        }
    }

    fn process_bme280(&mut self, my_elevation: f32) {
        if let (Some(t), Some(p), Some(h)) = self.bme280.read_sample().unwrap() {
            //println!("T= {:.2}  H = {:.2}, P = {:.2}", t, h, p);

            if self.last_indoor_sample.temperaturex10 != (t * 10.0) as i16 {
                self.last_indoor_sample.temperaturex10 = (t * 10.0) as i16;
                self.tx1
                    .send(Measurement::IndoorTemperature(self.convert_c_to_f(t) as i16))
                    .unwrap();
            }

            if self.last_indoor_sample.humidityx10 != (h * 10.0) as u16 {
                self.last_indoor_sample.humidityx10 = (h * 10.0) as u16;
                self.tx1.send(Measurement::IndoorHumidity(h as u8)).unwrap();
            }

            if self.last_indoor_sample.pressurex10 != (p * 10.0) as u32 {
                self.last_indoor_sample.pressurex10 = (p * 10.0) as u32;

                let atmospheric = p / 100.0;

                // This formula from https://github.com/adafruit/Adafruit_BME280_Library/blob/master/Adafruit_BME280.cpp line 465.
                let sea_level_hpa = atmospheric / f32::powf(1.0 - (my_elevation / 44330.0), 5.255);

                // The hpa_offset is a correction factor to obtain the correct sea level value.  The barometer reading at the
                // airport was used as a gold standard. Converted the airport inHg reading to hPa from
                // https://www.justintools.com/unit-conversion/pressure.php?k1=hectopascals&k2=inch-of-mercury
                // Took my sea_level_hpa reading and subtracted from the airport hpa reading to determine the offset needed.
                let hpa_offset = 1.0;
                let sea_level_hpa_compensated = sea_level_hpa - hpa_offset;

                //println!("atmospheric = {:.2}", atmospheric);
                //println!("sea level hpa = {}", sea_level_hpa);
                //println!("sea level hpa compensated = {}", sea_level_hpa_compensated);

                self.tx1
                    .send(Measurement::IndoorPressure(
                        self.convert_hpa_to_inHg(sea_level_hpa_compensated),
                    ))
                    .unwrap();
            }
        }
    }

    fn convert_c_to_f(&self, deg_c: f32) -> f32 {
        (deg_c * 1.8) + 32.0
    }

    fn convert_f_to_c(&self, deg_f: f32) -> f32 {
        (deg_f - 32.0) * 0.5556
    }

    #[allow(non_snake_case)]
    fn convert_hpa_to_inHg(&self, hpa: f32) -> f32 {
        hpa * 0.0295299830714
    }

    // Converts raw temperature to degrees fahrenheit
    fn convert_raw_temperature(&self, raw_temperature: u16) -> f32 {
        (raw_temperature as f32 / 10.0) - 40.0
    }

    // Converts raw wind speed to wind spped mph
    fn convert_raw_wind_speed(&self, raw_wind_speed: u8) -> u8 {
        match raw_wind_speed {
            0 => 0,
            _ => (((raw_wind_speed as f32 * 0.23) + 0.23) * 2.23694) as u8,
        }
    }

    // Converts raw rainbucket tips tip rainfall in 0.10 inches resolution
    fn convert_raw_rain_bucket_tips(&self, rain_bucket_tips: u16) -> f32 {
        rain_bucket_tips as f32 / 100.0
    }

    // Converts raw wind direction to a String
    fn convert_raw_wind_direction(&self, wind_direction: u8) -> String {
        const DIRECTION: [&'static str; 16] = [
            "NW", "WSW", "WNW", "W", "NNW", "SW", "N", "SSW", "ENE", "SE", "E", "ESE", "NE", "SSE",
            "NNE", "S",
        ];

        String::from(DIRECTION[wind_direction as usize])
    }

    fn calculate_outdoor_temperature(&self, raw_temperature: u16) -> OutdoorTemperature {
        let current_temperature_deg_f = self.convert_raw_temperature(raw_temperature);

        OutdoorTemperature {
            fahrenheit: (current_temperature_deg_f as i16),
            celsius: (self.convert_f_to_c(current_temperature_deg_f) as i16),
        }
    }

    // The heat index formula was taken from https://www.wpc.ncep.noaa.gov/html/heatindex_equation.shtml
    fn calculate_heat_index(&self, temperature_deg_f: f32, humidity: u8) -> HeatIndex {
        let t = temperature_deg_f;
        let rh = humidity as f32;
        let mut hi: f32;

        if t >= 80.0 {
            hi = -42.379 + 2.04901523 * t + 10.14333127 * rh
                - 0.22475541 * t * rh
                - 0.00683783 * t * t
                - 0.05481717 * rh * rh
                + 0.00122874 * t * t * rh
                + 0.00085282 * t * rh * rh
                - 0.00000199 * t * t * rh * rh;

            if t >= 80.0 && t <= 112.0 && rh < 13.0 {
                let adjustment = ((13.0 - rh) / 4.0) * ((17.0 - (t - 95.0).abs()) / 17.0).sqrt();
                hi -= -adjustment;
            }

            if t >= 80.0 && t <= 87.0 && rh > 85.0 {
                let adjustment = ((rh - 85.0) / 10.0) * ((87.0 - t) / 5.0);
                hi += adjustment;
            }
        } else {
            hi = 0.5 * (t + 61.0 + ((t - 68.0) * 1.2) + (rh * 0.094));
        };

        HeatIndex {
            fahrenheit: (hi.round() as i16),
            celsius: (self.convert_f_to_c(hi).round() as i16),
        }
    }

    // The dew point formula taken from https://www.omnicalculator.com/physics/dew-point
    // Accurate from -45 °C to 60 °C (-49F to 140F).
    fn calculate_dew_point(&self, temperature_deg_f: f32, humidity: u8) -> Dewpoint {
        let rh = humidity as f32;
        let t = temperature_deg_f;

        let alpha_t_rh = (rh / 100.0).ln() + ((17.625 * t) / (243.04 + t));
        let dp = (243.04 * alpha_t_rh) / (17.625 - alpha_t_rh);

        Dewpoint {
            fahrenheit: (dp.round() as i16),
            celsius: (self.convert_c_to_f(dp).round() as i16),
        }
    }

    fn calculate_wind_chill(&self, wind_speed_mph: u8, temperature_deg_f: f32) -> Windchill {
        let t = temperature_deg_f;
        let tt = f32::powf(wind_speed_mph as f32, 0.16);
        let wc = 35.74 + (0.6215 * t) - (35.75 * tt) + (0.4275 * t * tt);

        Windchill {
            fahrenheit: (wc.round() as i16),
            celsius: (self.convert_f_to_c(wc).round() as i16),
        }
    }

    fn update_average_wind_speed(&mut self) {
        let mut wind_speeds_mph_sum = 0u16;
        let items_collected = self.collected_wind_speeds_mph.len() as u16;

        while !self.collected_wind_speeds_mph.is_empty() {
            wind_speeds_mph_sum += self.collected_wind_speeds_mph.pop().unwrap() as u16;
        }

        let mut avg_wind_speed = 0;
        if items_collected != 0 {
            avg_wind_speed = wind_speeds_mph_sum / items_collected;
        }

        self.tx1
            .send(Measurement::AverageWindSpeedMph((avg_wind_speed) as u8))
            .unwrap();
    }
}
