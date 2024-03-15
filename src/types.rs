use esp32c3_hal::spi::{master::Spi, FullDuplexMode};
use ws2812_spi::Ws2812;

pub type SmartLedPeripheral = Ws2812<Spi<'static, esp32c3_hal::peripherals::SPI2, FullDuplexMode>>;
