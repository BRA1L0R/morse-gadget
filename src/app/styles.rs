use embedded_graphics::{
    mono_font::{ascii::FONT_5X7, MonoTextStyle},
    pixelcolor::BinaryColor,
};

pub const TEXT_STYLE: MonoTextStyle<'static, BinaryColor> =
    MonoTextStyle::new(&FONT_5X7, BinaryColor::On);
