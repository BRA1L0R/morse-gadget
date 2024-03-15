#![no_std]
#![no_main]

mod app;
mod events;
mod input;
pub mod module;
pub mod morse;
pub mod network;
mod reboot;
pub mod types;

extern crate alloc;

use alloc::boxed::Box;
use core::mem::MaybeUninit;
use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use esp32c3_hal::clock::{ClockControl, CpuClock};
use esp32c3_hal::embassy::executor::Executor;
use esp32c3_hal::gpio::AlternateFunction;
use esp32c3_hal::peripherals::{Peripherals, I2C0};
use esp32c3_hal::spi::master::Spi;
use esp32c3_hal::spi::SpiMode;
use esp32c3_hal::systimer::SystemTimer;
use esp32c3_hal::timer::TimerGroup;
use esp32c3_hal::{embassy, Rng, IO};
use esp32c3_hal::{i2c::I2C, peripherals::WIFI, prelude::*};
use module::BusModule;
use network::NetworkModule;
use types::SmartLedPeripheral;
use ws2812_spi::Ws2812;

use esp_backtrace as _;
use esp_wifi::{initialize, EspWifiInitFor, EspWifiInitialization};
use input::{InputModule, InputPins};

use crate::app::led_indicator::LedIndicator;
use crate::app::App;
use crate::events::Bus;
use crate::input::Input;
use crate::network::NetworkEvent;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

#[embassy_executor::task]
async fn main_task(app: Box<App>) {
    app.run().await;
}

fn run(
    i2c: I2C<'static, I2C0>,
    pins: InputPins,
    wifi: WIFI,
    wifi_token: EspWifiInitialization,
    pixel: SmartLedPeripheral,
) -> impl FnOnce(Spawner) {
    static INPUT_BUS: Bus<Input> = Channel::new();
    static NETWORK_BUS: Bus<NetworkEvent> = Channel::new();

    move |spawner| {
        let input_module = InputModule::init(&INPUT_BUS, pins).spawn(&spawner);
        let network_module = NetworkModule::init(&NETWORK_BUS, (wifi, wifi_token)).spawn(&spawner);
        let pixel = LedIndicator::new(pixel);

        let app = Box::new(App::init(i2c, input_module, network_module, pixel));
        spawner.spawn(main_task(app)).unwrap();
    }
}

#[entry]
fn main() -> ! {
    init_heap();

    let peripherals = Peripherals::take();
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    esp_println::logger::init_logger_from_env();

    // pressing sw4 at startup brings into bootloader
    let sw4 = io.pins.gpio18.into_floating_input();
    if sw4.is_input_high() {
        unsafe { reboot::reboot_download() };
    }

    // setup logger
    log::info!("Logger is setup");

    let executor = Box::new(Executor::new());
    let executor = Box::leak(executor); // ensure it exists forever

    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock160MHz).freeze();
    let systimer = SystemTimer::new(peripherals.SYSTIMER);

    log::info!("Running at {} clock speed", clocks.cpu_clock);
    log::info!("APB running at: {}", clocks.apb_clock);

    embassy::init(&clocks, TimerGroup::new(peripherals.TIMG0, &clocks));

    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio7,
        io.pins.gpio6,
        1_000_000u32.Hz(),
        &clocks,
    );

    let wifi_token: EspWifiInitialization = initialize(
        EspWifiInitFor::Wifi,
        systimer.alarm0,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    // let app = Box::new(App::init(i2c));

    // disable pullups that make sw3 not work
    peripherals.USB_DEVICE.conf0().write(|w| {
        w.usb_pad_enable()
            .clear_bit()
            .dm_pullup()
            .clear_bit()
            .dm_pulldown()
            .clear_bit()
            .dp_pullup()
            .clear_bit()
            .dp_pulldown()
            .clear_bit()
    });

    let mut sw3 = io.pins.gpio19.into_floating_input();
    sw3.set_alternate_function(AlternateFunction::Function0);

    let pins = InputPins {
        down: sw4.degrade(),
        left: sw3.degrade(),
        right: io.pins.gpio0.into_floating_input().degrade(),
        up: io.pins.gpio1.into_floating_input().degrade(),
    };

    let spi = Spi::new(peripherals.SPI2, 3_800_000u32.Hz(), SpiMode::Mode0, &clocks)
        .with_mosi(io.pins.gpio2);

    let neopixel: SmartLedPeripheral = Ws2812::new(spi);
    executor.run(run(i2c, pins, peripherals.WIFI, wifi_token, neopixel));
}
