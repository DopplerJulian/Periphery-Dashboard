use embassy_rp::{
    gpio::{Input, Output},
    peripherals::SPI1,
    spi::{self, Blocking, Spi},
};
use embassy_time::{Delay, Timer};
use embedded_graphics::prelude::*;

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
    frame: Display7in5,
    spi: ExclusiveDevice<Spi<'a, SPI1, Blocking>, Output<'a>, NoDelay>,
    sleeping: bool,
}
impl<'a> Display<'a> {
    pub async fn new(
        mut spi: ExclusiveDevice<Spi<'a, SPI1, Blocking>, Output<'a>, NoDelay>,
        busy_in: Input<'a>,
        dc: Output<'a>,
        rst: Output<'a>,
    ) -> Result<Self, DeviceError<spi::Error, core::convert::Infallible>> {
        // Setup EPD
        let epd = Epd7in5::new(&mut spi, busy_in, dc, rst, &mut Delay, None)?;

        // Use display graphics from embedded-graphics
        let mut display = Display7in5::default();
        display.clear(TriColor::White);

        Ok(Display {
            epd,
            frame: display,
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
        // Clear e-paper display's internal buffer
        self.epd.clear_frame(&mut self.spi, &mut Delay)?;
        // Fill the display white
        self.frame.clear(TriColor::White);
        // Update screen
        self.epd
            .update_and_display_frame(&mut self.spi, self.frame.buffer(), &mut Delay)
            .unwrap();
        // Let the display settle
        Timer::after_millis(3_000).await;
        Ok(())
    }
}
