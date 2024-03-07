use core::fmt::Display;

use embassy_time::Duration;
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    primitives::{PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StyledDrawable},
    Drawable,
};

pub struct MorseDisplay<'a> {
    morse_code: &'a [MorseCharacter],

    spacing_pixel: u32,
    empty_background: bool,

    position: Point,
}

impl<'a> MorseDisplay<'a> {
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

impl Drawable for MorseDisplay<'_> {
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

#[derive(PartialEq, Eq)]
pub enum MorseCharacter {
    Dot,
    Dash,
}

impl Display for MorseCharacter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MorseCharacter::Dot => f.write_str("."),
            MorseCharacter::Dash => f.write_str("-"),
        }
    }
}

const TRESHOLD: Duration = Duration::from_millis(200);

impl From<Duration> for MorseCharacter {
    fn from(value: Duration) -> Self {
        if value >= TRESHOLD {
            MorseCharacter::Dash
        } else {
            MorseCharacter::Dot
        }
    }
}

macro_rules! morse_char {
    // (...) =>  {
    //     MorseCharacter::Dot;
    //     MorseCharacter::Dot;
    //     MorseCharacter::Dot
    // };

    // (..) =>  {
    //     MorseCharacter::Dot;
    //     MorseCharacter::Dot
    // };
    (.) => {
        MorseCharacter::Dot
    };

    (-) => {
        MorseCharacter::Dash
    };
}

macro_rules! morse_expand {
    // ($($division:expr);+ , $($char:tt)+) =>  {

    // };

    ($($char:tt)+) =>  {
        &[$(morse_char!($char)),*]
    };
}

macro_rules! morse {
    ($( $let:expr =>  { $($char:tt)+ }),+) =>  {
        &[$(
            (
            morse_expand!($($char)*),
            $let
            )
        ),*]
    };
}

// returns None if input is bigger than 6
// pub fn to_legible(input: &[MorseCharacter]) -> Option<String<6>> {
//     use core::fmt::Write;

//     let mut buffer = String::new();
//     for char in input {
//         write!(&mut buffer, "{char}").ok()?;
//     }

//     Some(buffer)
// }

pub fn match_morse(input: &[MorseCharacter]) -> Option<char> {
    MORSE
        .iter()
        .find(|(morse, _)| morse == &input)
        .map(|(_, char)| char)
        .copied()
}

pub const MORSE: &[(&[MorseCharacter], char)] = morse! {
    'A' => { .- },       'B' => { -. . . },   'C' => { -.-. },
    'D' => { -. . },     'E' => { . },        'F' => { . .-. },
    'G' => { --. },      'H' => { . . . . },  'I' => { . . },
    'J' => { .--- },     'K' => { -.- },      'L' => { .-. . },
    'M' => { -- },       'N' => { -. },       'O' => { --- },
    'P' => { .--. },     'Q' => { --.- },     'R' => { .-. },
    'S' => { . . . },    'T' => { - },        'U' => { . .- },
    'V' => { . . .- },   'W' => { .-- },      'X' => { -. .- },
    'Y' => { -.-- },     'Z' => { --. . },

    '1' => { .---- },     '2' => {  .--- },     '3' => { . . .-- },
    '4' => { . . . .- },  '5' => { . . . . . }, '6' => { -. . . . },
    '7' => { --. . . },   '8' => { ---. . },    '9' => { ----. },
    '0' => { ----- },

    '&' => { .-. . . },   '@' => { .--.-. },    ':' => { ---. . . },
    ',' => { --. .-- },   '.' => { .-.-.- },    '\'' => { .----. },
    '\\' => { .-. .-. },  '?' => { . .--. . },  '/' => { -. .-. },
    '=' => { -. . .- },   '+' => { .-.-. },     '-' => { -. . . .- },
    '(' => { -.--. },     ')' => { -.--.- },
    '!' => { -.-.-- }
};
