use core::fmt::Display;

use embassy_time::Duration;

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
