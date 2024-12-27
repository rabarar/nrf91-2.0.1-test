#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let led1 = Output::new(p.P0_00, Level::Low, OutputDrive::Standard);
    let led2 = Output::new(p.P0_01, Level::Low, OutputDrive::Standard);
    let led3 = Output::new(p.P0_04, Level::Low, OutputDrive::Standard);
    let led4 = Output::new(p.P0_05, Level::Low, OutputDrive::Standard);

    let mut leds = [(led1, "one", 100u64), (led2, "two", 200u64), (led3, "three", 300u64), (led4, "four", 400u64)];

    loop {
        for (led, label, delay) in &mut leds  {
            led.set_high();
            defmt::info!("high: {} ", label);
            Timer::after_millis(*delay).await;
            led.set_low();
            defmt::info!("low: {} ", label);
            Timer::after_millis(*delay/2u64).await;
        }
    }
}

