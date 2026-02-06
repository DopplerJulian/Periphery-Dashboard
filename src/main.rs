#![no_std]
#![no_main]

mod bluetooth;
mod eink;

use embassy_executor::Spawner;
use embassy_rp as hal;
use embassy_rp::block::ImageDef;
use embassy_rp::gpio::{Level, Output};
use embassy_time::Timer;

//Panic Handler
use panic_probe as _;
// Defmt Logging
use defmt::info;
use defmt_rtt as _;

/// Tell the Boot ROM about our application
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: ImageDef = hal::block::ImageDef::secure_exe();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let _bt_controller = bluetooth::init_bluetooth_controller(
        p.PIN_23, p.PIN_25, p.PIO0, p.PIN_24, p.PIN_29, p.DMA_CH0, &spawner,
    );

    info!("Starting periphery_dashboard");

    let mut led = Output::new(p.PIN_15, Level::Low);
    loop {
        info!("Turning on LED");
        led.set_high(); // Turn on the LED
        Timer::after_millis(200).await;
        led.set_low(); // Turn off the LED
        Timer::after_millis(600).await;
    }
}

// Program metadata for `picotool info`.
// This isn't needed, but it's recommended to have these minimal entries.
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"periphery_dashboard"),
    embassy_rp::binary_info::rp_program_description!(c"your program description"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

// End of file
