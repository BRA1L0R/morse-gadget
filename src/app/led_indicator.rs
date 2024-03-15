use embassy_time::{block_for, Duration};
use smart_leds::{SmartLedsWrite, RGB8};

pub trait LedEffect {
    fn apply<L: SmartLedsWrite<Color = RGB8>>(self, led: &mut L) -> Result<(), L::Error>;
}

pub struct LedIndicator<P: SmartLedsWrite<Color = RGB8>> {
    led: P,
}

impl<P: SmartLedsWrite<Color = RGB8>> LedIndicator<P> {
    pub fn new(led: P) -> Self {
        Self { led }
    }

    pub fn play<E: LedEffect>(&mut self, effect: E) -> Result<(), P::Error> {
        effect.apply(&mut self.led)
    }
}

pub struct ChatNotificationEffect;

impl LedEffect for ChatNotificationEffect {
    fn apply<L: SmartLedsWrite<Color = RGB8>>(self, led: &mut L) -> Result<(), L::Error> {
        led.write([RGB8::new(138, 43, 226)])?;
        block_for(Duration::from_millis(50));
        led.write([RGB8::new(0, 0, 0)])?;

        block_for(Duration::from_millis(50));

        led.write([RGB8::new(138, 43, 226)])?;
        block_for(Duration::from_millis(50));
        led.write([RGB8::new(0, 0, 0)])?;

        Ok(())
    }
}
