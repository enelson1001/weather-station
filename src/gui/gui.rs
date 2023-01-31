use crossbeam_channel::Receiver;
use esp_idf_hal::{delay::FreeRtos, spi::SPI2};
use std::borrow::BorrowMut;

use crate::{
    gui::{
        display::{ADisplay, Display},
        views::{
            IndoorValueId, NavigationButtonId, OutdoorValueId, RainValueId, StatusValueId,
            TimeDateValueId, ViewId, Views, WindValueId,
        },
    },
    model::{
        peripherals::DisplaySpiPeripherals, scheduler::TimeDate, user_buttons::UserBtnState,
        weather_station::Measurement,
    },
};

pub struct Gui {
    cbc_rx_weather_station_measurements: Receiver<Measurement>,
    cbc_rx_user_btn: Receiver<UserBtnState>,
    cbc_rx_time_date: Receiver<TimeDate>,
    display: ADisplay,
    views: Views,
    view_showing: ViewId,
}

impl Gui {
    pub fn new(
        display_spi_peripherals: DisplaySpiPeripherals<SPI2>,
        rx1: Receiver<Measurement>,
        rx2: Receiver<UserBtnState>,
        rx3: Receiver<TimeDate>,
    ) -> Self {
        Self {
            cbc_rx_weather_station_measurements: rx1,
            cbc_rx_user_btn: rx2,
            cbc_rx_time_date: rx3,
            display: Display::build_display(display_spi_peripherals),
            views: Views::build_views(),
            view_showing: ViewId::IndoorOutdoor,
        }
    }

    pub fn start(mut self) {
        let _display_thread = std::thread::Builder::new()
            .stack_size(7000)
            .spawn(move || loop {
                println!("Starting Gui Thread");

                self.views
                    .show_view(self.display.borrow_mut(), ViewId::IndoorOutdoor)
                    .unwrap();

                loop {
                    self.check_for_time_events();
                    self.check_for_button_events();
                    self.check_for_weather_station_events();

                    FreeRtos::delay_ms(30);
                }
            });
    }

    fn check_for_time_events(&mut self) {
        if let Ok(time_date) = self.cbc_rx_time_date.try_recv() {
            match time_date {
                TimeDate::Time(time_str) => {
                    self.update_time_date_value(TimeDateValueId::Time as usize, &time_str);
                }

                TimeDate::Date(date_str) => {
                    self.update_time_date_value(TimeDateValueId::Date as usize, &date_str);
                }
            };
        }
    }

    fn check_for_button_events(&mut self) {
        if let Ok(user_btn_state) = self.cbc_rx_user_btn.try_recv() {
            match user_btn_state {
                UserBtnState::Btn1Pressed => {
                    self.show_button_pressed(NavigationButtonId::Previous as usize)
                }

                UserBtnState::Btn2Pressed => {
                    self.show_button_pressed(NavigationButtonId::Set as usize)
                }

                UserBtnState::Btn3Pressed => {
                    self.show_button_pressed(NavigationButtonId::Next as usize)
                }

                UserBtnState::Btn1Released => {
                    self.show_button_released(NavigationButtonId::Previous as usize);
                    self.show_previous_view();
                }
                UserBtnState::Btn2Released => {
                    self.show_button_released(NavigationButtonId::Set as usize)
                }

                UserBtnState::Btn3Released => {
                    self.show_button_released(NavigationButtonId::Next as usize);
                    self.show_next_view();
                }

                _ => (),
            };
        }
    }

    fn check_for_weather_station_events(&mut self) {
        while let Ok(measurement) = self.cbc_rx_weather_station_measurements.try_recv() {
            match measurement {
                Measurement::ChannelNumber(channel) => {
                    let value_str = &format!("{}", channel);
                    self.update_status_value(StatusValueId::Channel as usize, value_str);
                }

                Measurement::ProductId(product_id) => {
                    let value_str = &format!("{}", product_id);
                    self.update_status_value(StatusValueId::ProductId as usize, value_str);
                }

                Measurement::BatteryStatus(battery_status) => {
                    self.update_status_value(StatusValueId::Battery as usize, &battery_status)
                }

                Measurement::CurrentWindSpeedMph(current_wind_speed) => {
                    let value_str = &format!("{}{}", current_wind_speed, "mph");
                    self.update_wind_value(WindValueId::CurrentWindSpeed as usize, value_str);
                }

                Measurement::AverageWindSpeedMph(average_wind_speed) => {
                    let value_str = &format!("{}{}", average_wind_speed, "mph");
                    self.update_wind_value(WindValueId::AverageWindSpeed as usize, value_str);
                }

                Measurement::PeakWindSpeedMph(peak_wind_speed) => {
                    let value_str = &format!("{}{}", peak_wind_speed, "mph");
                    self.update_wind_value(WindValueId::PeakWindSpeed as usize, value_str);
                }

                Measurement::WindDirection(wind_direction) => {
                    //println!("Got new wind direction = {}", wind_direction);
                }

                Measurement::DailyRainfall(daily_rainfall) => {
                    let value_str = &format!("{:.2}{}", daily_rainfall, "in");
                    self.update_rain_value(RainValueId::DailyRainfall as usize, value_str);
                }

                Measurement::MonthlyRainfall(monthly_rainfall) => {
                    let value_str = &format!("{:.2}{}", monthly_rainfall, "in");
                    self.update_rain_value(RainValueId::MonthlyRainfall as usize, value_str);
                }

                Measurement::YearlyRainfall(yearly_rainfall) => {
                    let value_str = &format!("{:.2}{}", yearly_rainfall, "in");
                    self.update_rain_value(RainValueId::YearlyRainfall as usize, value_str);
                }

                Measurement::OutdoorTemperature(outdoor_temperature) => {
                    let value_str = &format!("{}{}", outdoor_temperature, "F");
                    self.update_outdoor_value(OutdoorValueId::Temperature as usize, value_str);
                }

                Measurement::OutdoorHumidity(outdoor_humidity) => {
                    let value_str = &format!("{}{}", outdoor_humidity, "%");
                    self.update_outdoor_value(OutdoorValueId::Humidity as usize, value_str);
                }

                Measurement::OutdoorHeatIndex(outdoor_heat_index) => {
                    let value_str = &format!("{}{}", outdoor_heat_index, "F");
                    self.update_outdoor_value(OutdoorValueId::HeatIndex as usize, value_str);
                }

                Measurement::OutdoorWindchill(outdoor_wind_chill) => {
                    let value_str = &format!("{}{}", outdoor_wind_chill, "F");
                    self.update_outdoor_value(OutdoorValueId::WindChill as usize, value_str);
                }

                Measurement::OutdoorDewpoint(outdoor_dew_point) => {
                    let value_str = &format!("{}{}", outdoor_dew_point, "F");
                    self.update_outdoor_value(OutdoorValueId::DewPoint as usize, value_str);
                }

                Measurement::IndoorTemperature(indoor_temperature) => {
                    let value_str = &format!("{}{}", indoor_temperature, "F");
                    self.update_indoor_value(IndoorValueId::Temperature as usize, value_str);
                }

                Measurement::IndoorHumidity(indoor_humidity) => {
                    let value_str = &format!("{}{}", indoor_humidity, "%");
                    self.update_indoor_value(IndoorValueId::Humidity as usize, value_str);
                }

                Measurement::IndoorPressure(indoor_pressure) => {
                    let value_str = &format!("{:.2}{}", indoor_pressure, " inHg");
                    self.update_indoor_value(IndoorValueId::Pressure as usize, value_str);
                }
            };
        }
    }

    fn update_status_value(&mut self, value_id: usize, value: &str) {
        self.views
            .wind_rain_status_view
            .status_panel
            .update_value(self.display.borrow_mut(), value_id, value)
            .unwrap();
    }

    fn update_wind_value(&mut self, value_id: usize, value: &str) {
        self.views
            .wind_rain_status_view
            .wind_panel
            .update_value(self.display.borrow_mut(), value_id, value)
            .unwrap();
    }

    fn update_rain_value(&mut self, value_id: usize, value: &str) {
        self.views
            .wind_rain_status_view
            .rain_panel
            .update_value(self.display.borrow_mut(), value_id, value)
            .unwrap();
    }

    fn update_outdoor_value(&mut self, value_id: usize, value: &str) {
        self.views
            .indoor_outdoor_view
            .outdoor_panel
            .update_value(self.display.borrow_mut(), value_id, value)
            .unwrap();
    }

    fn update_indoor_value(&mut self, value_id: usize, value: &str) {
        self.views
            .indoor_outdoor_view
            .indoor_panel
            .update_value(self.display.borrow_mut(), value_id, value)
            .unwrap();
    }

    fn update_time_date_value(&mut self, value_id: usize, value: &str) {
        self.views
            .time_date_view
            .time_date_panel
            .update_value(self.display.borrow_mut(), value_id, value)
            .unwrap();
    }

    fn show_next_view(&mut self) {
        self.views
            .hide_view(self.display.borrow_mut(), self.view_showing)
            .unwrap();

        match self.view_showing {
            ViewId::IndoorOutdoor => self.view_showing = ViewId::WindRainStatus,
            ViewId::WindRainStatus => self.view_showing = ViewId::TimeDate,
            ViewId::TimeDate => self.view_showing = ViewId::IndoorOutdoor,
        }

        self.views
            .show_view(self.display.borrow_mut(), self.view_showing)
            .unwrap();
    }

    fn show_previous_view(&mut self) {
        self.views
            .hide_view(self.display.borrow_mut(), self.view_showing)
            .unwrap();

        match self.view_showing {
            ViewId::IndoorOutdoor => self.view_showing = ViewId::TimeDate,
            ViewId::WindRainStatus => self.view_showing = ViewId::IndoorOutdoor,
            ViewId::TimeDate => self.view_showing = ViewId::WindRainStatus,
        }

        self.views
            .show_view(self.display.borrow_mut(), self.view_showing)
            .unwrap();
    }

    fn show_button_pressed(&mut self, button_id: usize) {
        let navigation_panel = match self.view_showing {
            ViewId::IndoorOutdoor => &mut self.views.indoor_outdoor_view.navigation_panel,
            ViewId::WindRainStatus => &mut self.views.wind_rain_status_view.navigation_panel,
            ViewId::TimeDate => &mut self.views.time_date_view.navigation_panel,
        };

        navigation_panel
            .show_button_pressed(self.display.borrow_mut(), button_id)
            .unwrap();
    }

    fn show_button_released(&mut self, button_id: usize) {
        let navigation_panel = match self.view_showing {
            ViewId::IndoorOutdoor => &mut self.views.indoor_outdoor_view.navigation_panel,
            ViewId::WindRainStatus => &mut self.views.wind_rain_status_view.navigation_panel,
            ViewId::TimeDate => &mut self.views.time_date_view.navigation_panel,
        };

        navigation_panel
            .show_button_released(self.display.borrow_mut(), button_id)
            .unwrap();
    }
}
