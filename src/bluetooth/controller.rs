use cyw43::{aligned_bytes, bluetooth::BtDriver};
use cyw43_pio::{PioSpi, RM2_CLOCK_DIVIDER};
use defmt::unwrap;
use embassy_executor::Spawner;
use embassy_rp::{
    Peri, bind_interrupts, dma,
    gpio::{Level, Output, Pin},
    peripherals::{DMA_CH0, PIO0},
    pio::{InterruptHandler, Pio, PioPin},
};
use static_cell::StaticCell;
use trouble_host::prelude::ExternalController;

pub async fn init(
    pwr_pin: Peri<'static, impl Pin>,
    cs_pin: Peri<'static, impl Pin>,
    pio_pin: Peri<'static, PIO0>,
    dio_pin: Peri<'static, impl PioPin>,
    clk_pin: Peri<'static, impl PioPin>,
    dma_pin: Peri<'static, DMA_CH0>,
    spawner: &Spawner,
) -> ExternalController<BtDriver<'static>, 10> {
    // Load cyw43 firmware
    let fw = aligned_bytes!("../../firmware/43439A0.bin");
    let btfw = aligned_bytes!("../../firmware/43439A0_btfw.bin");
    let clm = aligned_bytes!("../../firmware/43439A0_clm.bin");
    let nvram = aligned_bytes!("../../firmware/nvram_rp2040.bin");

    // Setup cyw43 controller
    let pwr = Output::new(pwr_pin, Level::Low);
    let cs = Output::new(cs_pin, Level::High);
    let mut pio = Pio::new(pio_pin, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        RM2_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        dio_pin,
        clk_pin,
        dma::Channel::new(dma_pin, Irqs),
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, bt_device, mut control, runner) =
        cyw43::new_with_bluetooth(state, pwr, spi, fw, btfw, nvram).await;
    spawner.spawn(unwrap!(cyw43_task(runner)));
    control.init(clm).await;

    let controller: ExternalController<_, 10> = ExternalController::new(bt_device);
    controller
}

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO0, 0>>>,
) -> ! {
    runner.run().await
}
