use alloc::format;
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    text::Text,
    Drawable,
};

use crate::morse::MorseCharacter;

use super::{chat::ChatMessage, styles::TEXT_STYLE};

pub struct ChatLogComponent<I> {
    messages: I,

    starting_px: Point,
    line_spacing: u32,
}

impl<I> ChatLogComponent<I> {
    pub fn new(messages: I, starting_px: Point) -> Self {
        Self {
            messages,
            starting_px,
            line_spacing: 0,
        }
    }

    pub fn line_spacing(self, line_spacing: u32) -> Self {
        Self {
            line_spacing,
            ..self
        }
    }
}

impl<'a, I> ChatLogComponent<I>
where
    I: Iterator<Item = &'a ChatMessage>,
{
    pub fn draw<D>(self, target: &mut D) -> Result<(), D::Error>
    where
        D: embedded_graphics::prelude::DrawTarget<Color = BinaryColor>,
    {
        let mut cursor = self.starting_px;

        for message in self.messages {
            let line = format!("{}: {}", message.from, message.text);
            Text::new(&line, cursor, TEXT_STYLE).draw(target)?;

            cursor.y -= 7 + self.line_spacing as i32 // 7 is font size
        }

        Ok(())
    }
}

/// Displayable morse-code buffer
pub struct MorseComponent<'a> {
    morse_code: &'a [MorseCharacter],

    spacing_pixel: u32,
    empty_background: bool,

    position: Point,
}

impl<'a> MorseComponent<'a> {
    pub fn new(morse_code: &'a [MorseCharacter], spacing_pixel: u32, position: Point) -> Self {
        Self {
            morse_code,
            spacing_pixel,
            position,
            empty_background: false,
        }
    }

    pub fn with_empty_background(self, background: bool) -> Self {
        Self {
            empty_background: background,
            ..self
        }
    }
}

impl Drawable for MorseComponent<'_> {
    type Color = BinaryColor;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: embedded_graphics::prelude::DrawTarget<Color = Self::Color>,
    {
        if self.empty_background && !self.morse_code.is_empty() {
            let mut calculated_empty: u32 = self
                .morse_code
                .iter()
                .map(|char| match char {
                    MorseCharacter::Dot => 2,
                    MorseCharacter::Dash => 5,
                })
                .map(|size| size + self.spacing_pixel)
                .sum();

            calculated_empty += self.spacing_pixel;
            let size = Size::new(calculated_empty, 2);

            let mut empty_position = self.position;
            empty_position.x -= self.spacing_pixel as i32;

            const BACKGROUND_STYLE: PrimitiveStyle<BinaryColor> = PrimitiveStyleBuilder::new()
                .fill_color(BinaryColor::Off)
                .build();

            Rectangle::new(empty_position, size).draw_styled(&BACKGROUND_STYLE, target)?;
        }

        let mut cursor = self.position;
        let style = PrimitiveStyleBuilder::new()
            .fill_color(BinaryColor::On)
            .stroke_color(BinaryColor::On)
            .build();

        for character in self.morse_code {
            let size = match character {
                MorseCharacter::Dot => Size::new(2, 2),
                MorseCharacter::Dash => Size::new(5, 2),
            };

            Rectangle::new(cursor, size).draw_styled(&style, target)?;
            cursor.x += (self.spacing_pixel + size.width) as i32;
        }

        Ok(())
    }
}
