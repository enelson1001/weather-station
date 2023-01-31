mod gui;
mod model;

use log::*;

use anyhow::{bail, Ok, Result};
use std::time::Duration;

//use ds323x::Ds323x;
use embedded_svc::wifi::Wifi;
use esp_idf_hal::{delay::FreeRtos, i2c::I2cDriver, peripheral};
use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_svc::{
    eventloop::{self, EspSystemEventLoop},
    netif::{EspNetif, EspNetifWait},
    nvs::EspDefaultNvsPartition,
    sntp,
    wifi::{EspWifi, WifiWait},
};

use crossbeam_channel::bounded;

use crate::{
    gui::gui::Gui,
    model::{
        acurite5n1::Acurite5n1,
        peripherals::{SystemPeripherals, RMT_RX_BUF_SIZE},
        scheduler::Scheduler,
        user_buttons::UserButtons,
        weather_station::WeatherStation,
    },
};

const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

fn main() -> Result<()> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = SystemPeripherals::take();
    let sysloop = eventloop::EspSystemEventLoop::take()?;
    let _wifi = wifi(peripherals.modem, sysloop.clone())?;

    let sntp = sntp::EspSntp::new_default()?;
    while sntp.get_sync_status() != sntp::SyncStatus::Completed {
        FreeRtos::delay_ms(100);
    }
    info!("SNTP initialized");

    // Create Crossbeam channels
    let (tx1, rx1) = bounded(5); // tx = Acurite5n1,     rx = WeatherStation
    let (tx2, rx2) = bounded(40); // tx = WeatherStation rx = Gui
    let (tx3, rx3) = bounded(5); // tx = UserButtons     rx = Gui
    let (tx4, rx4) = bounded(2); // tx = Scheduler       rx = Gui
    let (tx5, rx5) = bounded(5); // tx = Scheduler       rx = WeatherStation

    // Create the I2C devices
    //let mut rtc = RealTimeClock::new(Ds323x::new_ds3231(i2c_bus_manager.acquire_i2c()));

    use std::result::Result::Ok;
    let i2c_bus_manager: &'static _ =
        shared_bus::new_std!(I2cDriver = peripherals.i2c0_driver).unwrap();
    let i2c0_proxy_1 = i2c_bus_manager.acquire_i2c();

    // Create user buttons
    let user_buttons = UserButtons::new(peripherals.buttons, tx3);

    // Create the Acurite5n1 weather sensor
    let acurite5n1 = Acurite5n1::new(tx1, peripherals.rx_rmt_driver, RMT_RX_BUF_SIZE);

    // Create the weather station
    let weather_station = WeatherStation::new(i2c0_proxy_1, rx1, rx5, tx2);

    // Create the Gui
    let gui = Gui::new(peripherals.display, rx2, rx3, rx4);

    // Create the Scheduler
    let scheduler = Scheduler::new(tx4, tx5);

    // Start the threads
    user_buttons.start();
    weather_station.start();
    gui.start();
    FreeRtos::delay_ms(30);
    acurite5n1.start();
    scheduler.start();

    //#[cfg_attr(link_section = ".rtc.data.rtc_memory" )]
    //static mut YOUR_RTC_DATA_STRUCT: rtc_data = rtc_data::new();

    loop {
        FreeRtos::delay_ms(1000);
    }
}

fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    use std::net::Ipv4Addr;
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let mut esp_wifi = Box::new(EspWifi::new(modem, sysloop.clone(), Some(nvs))?);

    esp_wifi.set_configuration(&embedded_svc::wifi::Configuration::Client(
        embedded_svc::wifi::ClientConfiguration {
            ssid: WIFI_SSID.into(),
            password: WIFI_PASSWORD.into(),
            ..Default::default()
        },
    ))?;

    esp_wifi.start().unwrap();
    info!("Starting wifi");

    if !WifiWait::new(&sysloop)
        .unwrap()
        .wait_with_timeout(Duration::from_secs(20), || esp_wifi.is_started().unwrap())
    {
        bail!("Wifi did not start");
    }

    info!("Connecting wifi ...");

    esp_wifi.connect().unwrap();

    if !EspNetifWait::new::<EspNetif>(esp_wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(45),
        || {
            esp_wifi.is_connected().unwrap()
                && esp_wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        bail!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = esp_wifi.sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(esp_wifi)
}
