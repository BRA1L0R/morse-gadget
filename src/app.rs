pub mod chat;
pub mod components;
pub mod led_indicator;
pub mod styles;

use core::str::FromStr;

use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Instant};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
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
    app::{
        components::{ChatLogComponent, MorseComponent},
        styles::TEXT_STYLE,
    },
    input::{Direction, Input, InputModule},
    module::WithBus,
    morse::{match_morse, MorseCharacter},
    network::{NetworkEvent, NetworkMessage, NetworkModule},
    reboot::reboot_download,
    types::SmartLedPeripheral,
};

use self::{
    chat::ChatLog,
    led_indicator::{ChatNotificationEffect, LedIndicator},
};

pub struct App {
    display: ssd1306::Ssd1306<
        I2CInterface<I2C<'static, I2C0>>,
        DisplaySize128x64,
        BufferedGraphicsMode<DisplaySize128x64>,
    >,

    network_module: WithBus<NetworkModule>,
    input_module: WithBus<InputModule>,
    led: LedIndicator<SmartLedPeripheral>,

    input: String<16>,
    morse_buffer: Vec<MorseCharacter, 6>,
    chat_log: ChatLog,

    typing_indicator: Option<Instant>,
}

impl App {
    pub fn init(
        i2c: I2C<'static, I2C0>,
        input_module: WithBus<InputModule>,
        network_module: WithBus<NetworkModule>,
        led: LedIndicator<SmartLedPeripheral>,
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
            led,

            morse_buffer: Vec::new(),
            chat_log: ChatLog::new(),
            typing_indicator: None,
        }
    }

    async fn input_logic(&mut self, input: Input) {
        match input.direction {
            Direction::Right if self.morse_buffer.is_empty() && !self.input.is_empty() => {
                let buffer = core::mem::replace(&mut self.input, String::new());

                self.chat_log.push_message(chat::From::You, buffer.clone());
                self.network_module
                    .send_message(NetworkMessage::Text(buffer))
                    .await;

                // sent message: not typing
                self.network_module
                    .send_message(NetworkMessage::Typing(false))
                    .await;
            }
            Direction::Right => {
                let match_morse = match_morse(&self.morse_buffer);
                let Some(character) = match_morse else { return }; // todo: tell user that the morse char is wrong

                self.morse_buffer.clear();
                self.input.push(character).ok();

                // someone is typing!
                self.network_module
                    .send_message(NetworkMessage::Typing(true))
                    .await;
            }
            Direction::Down => {
                let character = MorseCharacter::from(input.duration);
                self.morse_buffer.push(character).ok();
            }
            Direction::Left => {
                // pop character off morse buffer
                let pop = self.morse_buffer.pop();
                let None = pop else { return };

                // if nothing is being written in morse it means user wants to delete text
                self.input.pop();

                // when user starts deleting text instead of morse send a typing packet
                // saying it's not typing anymore
                self.network_module
                    .send_message(NetworkMessage::Typing(false))
                    .await;
            }
            Direction::Up if input.duration >= Duration::from_secs(1) => unsafe {
                reboot_download()
            },

            _ => (),
        }
    }

    async fn process_network(&mut self, event: NetworkEvent) {
        match event.message {
            NetworkMessage::Text(text) => {
                self.chat_log.push_message(chat::From::Other, text);
                self.led.play(ChatNotificationEffect).unwrap();
            }
            NetworkMessage::Typing(is_typing) => {
                self.typing_indicator = is_typing.then(|| Instant::now())
            }
            NetworkMessage::Ping | NetworkMessage::Pong => {
                if matches!(event.message, NetworkMessage::Ping) {
                    self.network_module.send_message(NetworkMessage::Pong).await;
                }

                self.chat_log
                    .push_message(chat::From::System, String::from_str("Online!").unwrap());
            }
        }
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
        let is_input_shown = !self.input.is_empty() || !self.morse_buffer.is_empty();
        if is_input_shown {
            Rectangle::new(
                Point::new(0, DISPLAY_HEIGHT - TEXT_BOX_SIZE as i32),
                Size::new(128, TEXT_BOX_SIZE),
            )
            .into_styled(BORDER_STYLE)
            .draw(&mut self.display)
            .unwrap();
        }

        // Text::new(&self.input, Point::new(2, 60), ;
        Text::new(&self.input, Point::new(2, DISPLAY_HEIGHT - 3), TEXT_STYLE)
            .draw(&mut self.display)
            .unwrap();

        MorseComponent::new(&self.morse_buffer, 3, Point::new(60, DISPLAY_HEIGHT - 2))
            .with_empty_background(true)
            .draw(&mut self.display)
            .unwrap();

        let chat_log_pos = match is_input_shown {
            true => Point::new(0, DISPLAY_HEIGHT - TEXT_BOX_SIZE as i32 - 2),
            false => Point::new(0, DISPLAY_HEIGHT - 2),
        };

        ChatLogComponent::new(self.chat_log.messages(), chat_log_pos)
            .line_spacing(1)
            .draw(&mut self.display)
            .unwrap();

        const TYPING_MAX: u64 = 10;
        if let Some(..=TYPING_MAX) = self
            .typing_indicator
            .map(|started| Instant::now() - started)
            .map(|duration| duration.as_secs())
        {
            Text::new("typing...", Point::new(75, 5), TEXT_STYLE)
                .draw(&mut self.display)
                .unwrap();
        }

        self.display.flush().unwrap();
    }

    pub async fn run(mut self) -> ! {
        self.network_module
            // notify everyone of our presence
            .send_message(NetworkMessage::Ping)
            .await;

        loop {
            self.draw();

            let event = select(
                self.input_module.receive_event(),
                self.network_module.receive_event(),
            )
            .await;

            match event {
                Either::First(input) => self.input_logic(input).await,
                Either::Second(network) => self.process_network(network).await,
            }
        }
    }
}
