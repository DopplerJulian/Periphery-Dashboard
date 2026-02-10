use defmt::info;
use embassy_time::{Delay, Timer};
use embedded_graphics::{
    prelude::*,
    primitives::{Line, PrimitiveStyle},
};
use embedded_hal::{
    digital::{InputPin, OutputPin},
    spi::SpiDevice,
};
use epd_waveshare::{epd7in5b_v2::*, prelude::*};

pub async fn init_display<SPI, BUSY, DC, RST>(
    mut spi: SPI,
    busy_in: BUSY,
    dc: DC,
    rst: RST,
) -> Result<(), SPI::Error>
where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
{
    // Setup EPD
    let mut epd = Epd7in5::new(&mut spi, busy_in, dc, rst, &mut Delay, None)?;

    // Use display graphics from embedded-graphics
    let mut display = Display7in5::default();
    display.clear(TriColor::White);

    info!("starting display test");
    // Use embedded graphics for drawing a line
    let _ = Line::new(Point::new(0, 120), Point::new(0, 295))
        .into_styled(PrimitiveStyle::with_stroke(TriColor::Black, 1))
        .draw(&mut display);

    // Display updated frame
    epd.update_frame(&mut spi, display.buffer(), &mut Delay)?;
    epd.display_frame(&mut spi, &mut Delay)?;

    // Set the EPD to sleep
    epd.sleep(&mut spi, &mut Delay)?;
    info!("now sleeping for 10 seconds");
    Timer::after_millis(10000).await;

    epd.wake_up(&mut spi, &mut Delay)?;
    epd.clear_frame(&mut spi, &mut Delay)?;
    display.clear(TriColor::White).unwrap();
    epd.update_and_display_frame(&mut spi, display.buffer(), &mut Delay)
        .unwrap();

    info!("cleared frame");

    Ok(())
}
