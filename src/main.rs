const N_PIXELS: usize = 5;
const N_BITS: usize = N_PIXELS * 3;

use std::time::Duration;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        gpio::{OutputPin, PinDriver},
        peripheral::Peripheral,
        peripherals::Peripherals,
        rmt::{
            config::TransmitConfig, FixedLengthSignal, PinState, Pulse, RmtChannel, TxRmtDriver,
        },
    },
    sys::EspError,
};
use rgb::{ComponentSlice, RGB8};

fn main() -> Result<(), EspError> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let _sysloop = EspSystemEventLoop::take()?;
    log::info!("Hello, world!");

    // let mut leds = WS2812B::new(peripherals.pins.gpio2, peripherals.rmt.channel0).unwrap();
    // let mut colors = [
    //     RGB8::new(255, 0, 0),
    //     RGB8::new(0, 255, 0),
    //     RGB8::new(0, 0, 255),
    //     RGB8::new(255, 0, 0),
    //     RGB8::new(0, 255, 0),
    // ];
    let mut pin = PinDriver::output(peripherals.pins.gpio2).unwrap();
    pin.set_low();
    std::thread::sleep(Duration::from_secs(1));
    loop {
        // leds.set_pixels(&colors).unwrap();
        // colors.rotate_left(1);
        // std::thread::sleep(Duration::from_secs(1));

        for _ in (0..24) {
            pin.set_high();
            std::thread::sleep(Duration::from_nanos(700));

            pin.set_low();
            std::thread::sleep(Duration::from_nanos(600));
        }
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub struct WS2812B<'a> {
    tx_rtm_driver: TxRmtDriver<'a>,
}

impl<'a> WS2812B<'a> {
    // Rust ESP Board gpio2,  ESP32-C3-DevKitC-02 gpio8
    pub fn new(
        led: impl Peripheral<P = impl OutputPin> + 'a,
        channel: impl Peripheral<P = impl RmtChannel> + 'a,
    ) -> Result<Self, EspError> {
        let config = TransmitConfig::new().clock_divider(2);
        let tx = TxRmtDriver::new(channel, led, &config)?;
        Ok(Self { tx_rtm_driver: tx })
    }

    pub fn set_pixels(&mut self, rgb: &[RGB8; N_PIXELS]) -> Result<(), EspError> {
        // let color: u32 = ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | rgb.b as u32;
        let ticks_hz = self.tx_rtm_driver.counter_clock()?;

        let t0h = Pulse::new_with_duration(ticks_hz, PinState::High, &ns(350))?;
        let t0l = Pulse::new_with_duration(ticks_hz, PinState::Low, &ns(800))?;
        let t1h = Pulse::new_with_duration(ticks_hz, PinState::High, &ns(700))?;
        let t1l = Pulse::new_with_duration(ticks_hz, PinState::Low, &ns(600))?;
        let mut signal = FixedLengthSignal::<N_BITS>::new();

        let pulses = rgb
            .iter()
            .flat_map(|rgb| rgb.as_slice().iter())
            .flat_map(|b| (0..8).map(move |offset| ((b & (1 << offset)) >> offset) == 1))
            .map(|v| match v {
                true => (t1h, t1l),
                false => (t0h, t0l),
            });

        for (idx, pulse) in pulses.enumerate() {
            signal.set((N_PIXELS * 24) - idx, &pulse)?;
        }

        /*
        for i in (0..24).rev() {
            let p = 2_u32.pow(i);
            let bit = p & color != 0;
            let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
            signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
        }
        */
        self.tx_rtm_driver.start_blocking(&signal)?;

        Ok(())
    }
}

fn ns(nanos: u64) -> Duration {
    Duration::from_nanos(nanos)
}
