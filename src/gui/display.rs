use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::{DrawTarget, RgbColor},
};
use mipidsi::{models::ILI9342CRgb565, Builder, ColorOrder, Orientation};

use display_interface_spi::SPIInterfaceNoCS;

use esp_idf_hal::{
    delay::Ets,
    gpio::{AnyOutputPin, DriveStrength, Gpio19, Output, PinDriver},
    spi::{Dma, SpiConfig, SpiDeviceDriver, SpiDriver, SPI2},
    units::FromValueType,
};

use crate::model::peripherals::DisplaySpiPeripherals;

pub type ADisplay = mipidsi::Display<
    SPIInterfaceNoCS<
        SpiDeviceDriver<'static, SpiDriver<'static>>,
        PinDriver<'static, AnyOutputPin, Output>,
    >,
    ILI9342CRgb565,
    PinDriver<'static, AnyOutputPin, Output>,
>;

pub struct Display {}

impl Display {
    pub fn build_display(
        display_spi_peripherals: DisplaySpiPeripherals<SPI2>,
    ) -> ADisplay {
        if let Some(backlight) = display_spi_peripherals.control.backlight {
            //let mut backlight = PinDriver::output(backlight)?;
            let mut backlight = PinDriver::output(backlight).unwrap();

            backlight.set_drive_strength(DriveStrength::I40mA).unwrap();
            //backlight.set_high()?;
            backlight.set_high().unwrap();
            std::mem::forget(backlight); // TODO: For now
        }

        let mut delay = Ets;

        let spi = SpiDeviceDriver::new_single(
            display_spi_peripherals.spi,
            display_spi_peripherals.sclk,
            display_spi_peripherals.sdo,
            Option::<Gpio19>::None,
            Dma::Channel1(4092),
            display_spi_peripherals.cs,
            &SpiConfig::new().write_only(true).baudrate(26.MHz().into()),
        )
        .unwrap();

        let rst = PinDriver::output(display_spi_peripherals.control.rst).unwrap();
        let dc = PinDriver::output(display_spi_peripherals.control.dc).unwrap();
        let di = display_interface_spi::SPIInterfaceNoCS::new(spi, dc);

        let mut display = Builder::ili9342c_rgb565(di)
            .with_display_size(320, 240)
            .with_color_order(ColorOrder::Bgr)
            .with_orientation(Orientation::Portrait(false))
            .with_invert_colors(true)
            .init(&mut delay, Some(rst))
            .unwrap();

        display.clear(Rgb565::BLACK).unwrap();

        display
    }
}
