use defmt::info;
use embassy_rp::{
    gpio::{Input, Output},
    peripherals::SPI1,
    spi::{self, Blocking, Spi},
};
use embassy_time::Delay;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    prelude::*,
    text::{Baseline, Text},
};

use embedded_hal_bus::spi::{DeviceError, ExclusiveDevice, NoDelay};
use epd_waveshare::{epd7in5b_v2::*, prelude::*};

pub struct Display<'a> {
    epd: Epd7in5<
        ExclusiveDevice<Spi<'a, SPI1, Blocking>, Output<'a>, NoDelay>,
        Input<'a>,
        Output<'a>,
        Output<'a>,
        Delay,
    >,
    display: Display7in5,
    spi: ExclusiveDevice<Spi<'a, SPI1, Blocking>, Output<'a>, NoDelay>,
    sleeping: bool,
}
impl<'a> Display<'a> {
    pub fn new(
        mut spi: ExclusiveDevice<Spi<'a, SPI1, Blocking>, Output<'a>, NoDelay>,
        busy_in: Input<'a>,
        dc: Output<'a>,
        rst: Output<'a>,
    ) -> Result<Self, DeviceError<spi::Error, core::convert::Infallible>> {
        info!("setting up display");
        // Setup EPD
        let mut epd = Epd7in5::new(&mut spi, busy_in, dc, rst, &mut Delay, None).unwrap();
        epd.set_background_color(TriColor::White);

        info!("epd created");

        // Use display graphics from embedded-graphics
        let mut display = Display7in5::default();
        display.clear(TriColor::White);

        info!("display created");

        Ok(Display {
            epd,
            display,
            spi,
            sleeping: false,
        })
    }

    pub async fn clear(
        &mut self,
    ) -> Result<(), DeviceError<spi::Error, core::convert::Infallible>> {
        if self.sleeping {
            self.epd.wake_up(&mut self.spi, &mut Delay)?;
            self.sleeping = false;
        }
        // Fill the display white
        self.display.clear(TriColor::White);
        // Clear e-paper display's buffer
        self.epd.clear_frame(&mut self.spi, &mut Delay)?;
        self.epd.wait_until_idle(&mut self.spi, &mut Delay)?;
        Ok(())
    }

    pub fn display_text(
        &mut self,
    ) -> Result<(), DeviceError<spi::Error, core::convert::Infallible>> {
        let text_style = MonoTextStyleBuilder::new()
            .font(&FONT_10X20)
            .text_color(TriColor::Black)
            .build();

        Text::with_baseline("Test", Point::new(100, 100), text_style, Baseline::Top)
            .draw(&mut self.display)
            .unwrap();

        self.epd
            .update_and_display_frame(&mut self.spi, self.display.buffer(), &mut Delay)
            .unwrap();
        Ok(())
    }
}
