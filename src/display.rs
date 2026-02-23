use defmt::info;
use embassy_rp::{
    gpio::{Input, Output},
    peripherals::SPI1,
    spi::{self, Blocking, Spi},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Delay;
use embedded_graphics::{
    mono_font::{MonoTextStyleBuilder, ascii::FONT_10X20},
    prelude::*,
    primitives::Rectangle,
    text::{Baseline, Text},
};
use embedded_hal_bus::spi::{DeviceError, ExclusiveDevice, NoDelay};
use epd_waveshare::{epd7in5b_v2::*, prelude::*};

use heapless::Vec;

pub static DISPLAY: Mutex<CriticalSectionRawMutex, Option<Display>> = Mutex::new(None);

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
        info!("cleared Display");
        Ok(())
    }

    pub fn write_to_buffer(
        &mut self,
        values: &[u8; 32],
        cursor: u32,
    ) -> Result<(), DeviceError<spi::Error, core::convert::Infallible>> {
        let colors = bytes_to_color(values);
        let display_width = 800u32;
        let pixel_cursor = cursor * 128;

        if (pixel_cursor % display_width) + 128 > display_width {
            let first_len = display_width - (pixel_cursor % display_width);

            let area_1 = Rectangle::new(
                Point {
                    x: (pixel_cursor % display_width) as i32,
                    y: (pixel_cursor / display_width) as i32,
                },
                Size {
                    width: first_len,
                    height: 1,
                },
            );
            let area_2 = Rectangle::new(
                Point {
                    x: 0 as i32,
                    y: (pixel_cursor / display_width + 1) as i32,
                },
                Size {
                    width: 128 - first_len,
                    height: 1,
                },
            );
            // There is most likely a better way, but this should suffice for now
            let first: Vec<TriColor, 128> = Vec::from_slice(&colors[..first_len as usize]).unwrap();
            let second: Vec<TriColor, 128> =
                Vec::from_slice(&colors[(first_len as usize)..]).unwrap();
            self.display.fill_contiguous(&area_1, first);
            self.display.fill_contiguous(&area_2, second);
        } else {
            let area = Rectangle::new(
                Point {
                    x: (pixel_cursor % display_width) as i32,
                    y: (pixel_cursor / display_width) as i32,
                },
                Size {
                    width: 128,
                    height: 1,
                },
            );
            self.display.fill_contiguous(&area, colors);
        }
        Ok(())
    }

    pub fn display_buffer(&mut self) {
        self.epd
            .update_and_display_frame(&mut self.spi, self.display.buffer(), &mut Delay)
            .unwrap();
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

fn bytes_to_color(bytes: &[u8; 32]) -> [TriColor; 128] {
    let mut result = [TriColor::White; 128];
    for i in 0usize..16 {
        let i = i * 2;

        for j in 0u8..8 {
            let black = bytes[i] >> j & 1;
            let color = bytes[i + 1] >> j & 1;

            if black == 1 && color == 1 {
            } else {
                result[i * 8 + (j as usize)] = if black == 0 {
                    TriColor::Black
                } else {
                    TriColor::Chromatic
                };
            }
        }
    }

    result
}
