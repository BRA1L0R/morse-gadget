use embassy_futures::select::select;
use embassy_time::Duration;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::{ascii::FONT_5X7, MonoTextStyle},
    pixelcolor::BinaryColor,
    primitives::{Primitive, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
    Drawable,
};
use esp32c3_hal::i2c::I2C;
use heapless::{String, Vec};
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::I2CInterface,
    rotation::DisplayRotation,
    size::DisplaySize128x64,
    Ssd1306,
};

use esp32c3_hal::peripherals::I2C0;

use crate::{
    input::{Direction, Input, InputModule},
    morse::{match_morse, MorseCharacter, MorseDisplay},
    network::{NetworkDevice, NetworkEvent},
    reboot::reboot_download,
};

pub struct App {
    display: ssd1306::Ssd1306<
        I2CInterface<I2C<'static, I2C0>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,

    network_module: NetworkDevice,
    input_module: InputModule,

    input: String<16>,
    morse_buffer: Vec<MorseCharacter, 6>,
}

impl App {
    pub fn init(
        i2c: I2C<'static, I2C0>,
        input_module: InputModule,
        network_module: NetworkDevice,
    ) -> Self {
        let mut display = Ssd1306::new(
            I2CInterface::new(i2c, 0x3c, 0x40),
            DisplaySize128x64,
            DisplayRotation::Rotate0,
        )
        .into_buffered_graphics_mode();

        display.init().unwrap();

        Self {
            display,
            input: String::new(),
            input_module,
            network_module,

            morse_buffer: Vec::new(),
        }
    }

    fn process_input(&mut self, input: Input) {
        match input.direction {
            Direction::Right if self.morse_buffer.is_empty() => {
                // self.network_module.

                self.input.clear();
            }
            Direction::Right => {
                let Some(character) = match_morse(&self.morse_buffer) else {
                    // todo
                    return;
                };

                self.morse_buffer.clear();
                self.input.push(character).ok();
            }
            Direction::Down => {
                let character = MorseCharacter::from(input.duration);
                self.morse_buffer.push(character).ok();
            }
            Direction::Left => {
                // pop character off morse buffer
                let pop = self.morse_buffer.pop();

                // if nothing is being written in morse it means user wants to delete text
                if pop.is_none() {
                    self.input.pop();
                }
            }
            Direction::Up if input.duration >= Duration::from_secs(1) => unsafe {
                reboot_download()
            },

            _ => (),
        }
    }

    fn process_network(&mut self, event: NetworkEvent) {
        // log::info!("{event:?}");
    }

    pub fn draw(&mut self) {
        const DISPLAY_HEIGHT: i32 = 64;

        self.display.clear(BinaryColor::Off).unwrap();

        // base box
        const BORDER_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(1)
            .fill_color(BinaryColor::Off)
            .build();

        const TEXT_BOX_SIZE: u32 = 10;

        // only draw input box if there's morse input or text in the buffer
        if !self.input.is_empty() || !self.morse_buffer.is_empty() {
            Rectangle::new(
                Point::new(0, DISPLAY_HEIGHT - TEXT_BOX_SIZE as i32),
                Size::new(128, TEXT_BOX_SIZE),
            )
            .into_styled(BORDER_STYLE)
            .draw(&mut self.display)
            .unwrap();
        }

        // Text::new(&self.input, Point::new(2, 60), ;
        let style = MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
        Text::new(&self.input, Point::new(2, DISPLAY_HEIGHT - 3), style)
            .draw(&mut self.display)
            .unwrap();

        MorseDisplay::new(&self.morse_buffer, 3, Point::new(60, DISPLAY_HEIGHT - 2))
            .with_empty_background(true)
            .draw(&mut self.display)
            .unwrap();

        self.display.flush().unwrap();
    }

    pub async fn run(self) -> ! {
        loop {
            // select(self.input_module.event(), self.network_module.event())
        }
    }
}
