use esp_idf_hal::{
    gpio::{AnyInputPin, AnyOutputPin,},
    i2c::{I2cConfig, I2cDriver},
    modem::Modem,
    peripherals::Peripherals,
    rmt::{RmtReceiveConfig, RxRmtDriver},
    spi::SPI2,
    units::FromValueType,
};

pub const RMT_RX_BUF_SIZE: usize = 1024;  //was 8192

pub struct ButtonsPeripherals {
    pub left_button: AnyInputPin,
    pub middle_button: AnyInputPin,
    pub right_button: AnyInputPin,
}

pub struct DisplayControlPeripherals {
    pub backlight: Option<AnyOutputPin>,
    pub dc: AnyOutputPin,
    pub rst: AnyOutputPin,
}

pub struct DisplaySpiPeripherals<SPI> {
    pub control: DisplayControlPeripherals,
    pub spi: SPI,
    pub sclk: AnyOutputPin,
    pub sdo: AnyOutputPin,
    pub cs: Option<AnyOutputPin>,
}

pub struct TestPointsPeripherals {
    pub test_point_one: AnyOutputPin,
}

pub struct SystemPeripherals<SPI> {
    pub rx_rmt_driver: RxRmtDriver<'static>,
    pub i2c0_driver: I2cDriver<'static>,
    pub display: DisplaySpiPeripherals<SPI>,
    pub buttons: ButtonsPeripherals,
    pub modem: Modem,
    pub test_points: TestPointsPeripherals,
}

impl SystemPeripherals<SPI2> {
    pub fn take() -> Self {
        let peripherals = Peripherals::take().unwrap();

        let rmt_input = peripherals.pins.gpio5;

        // Create the RMT Receiver interface
        let rx_rmt_driver = RxRmtDriver::new(
            peripherals.rmt.channel0,
            rmt_input,
            &RmtReceiveConfig::new().idle_threshold(700u16).mem_block_num(8),  // will ignore pulses longer than this
            RMT_RX_BUF_SIZE,
        )
        .unwrap();

        // Create the I2cDriver for the DS3231 RTC, AT24C32 32K EEPROM, Bme280
        let i2c0 = peripherals.i2c0;
        let sda = peripherals.pins.gpio21;
        let scl = peripherals.pins.gpio22;
        let config = I2cConfig::new().baudrate(400.kHz().into());
        let i2c0_driver = I2cDriver::new(i2c0, sda, scl, &config).unwrap();

        Self {
            rx_rmt_driver,

            i2c0_driver,

            display: DisplaySpiPeripherals {
                control: DisplayControlPeripherals {
                    backlight: Some(peripherals.pins.gpio32.into()),
                    dc: peripherals.pins.gpio27.into(),
                    rst: peripherals.pins.gpio33.into(),
                },
                spi: peripherals.spi2,
                sclk: peripherals.pins.gpio18.into(),
                sdo: peripherals.pins.gpio23.into(),
                cs: Some(peripherals.pins.gpio14.into()),
            },

            buttons: ButtonsPeripherals {
                left_button: peripherals.pins.gpio39.into(),
                middle_button: peripherals.pins.gpio38.into(),
                right_button: peripherals.pins.gpio37.into(),
            },

            modem: peripherals.modem,

            test_points: TestPointsPeripherals {
                test_point_one: peripherals.pins.gpio2.into(),
            },
        }
    }
}
