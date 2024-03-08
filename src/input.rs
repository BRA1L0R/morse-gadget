use embassy_futures::select::Either4;
use embassy_time::{Duration, Instant};
use embedded_hal_async::digital::Wait;
use esp32c3_hal::gpio::{AnyPin, Floating, Input as GpioInput};

use crate::{
    events,
    module::{BusModule, Spawnable, WithBus},
};

type InputPin = AnyPin<GpioInput<Floating>>;

pub struct InputPins {
    pub down: InputPin,
    pub up: InputPin,
    pub left: InputPin,
    pub right: InputPin,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub struct Input {
    pub duration: Duration,
    pub direction: Direction,
}

impl InputPins {
    async fn wait_for_any(&mut self) -> Input {
        // embassy
        let button = embassy_futures::select::select4(
            self.down.wait_for_rising_edge(),
            self.up.wait_for_rising_edge(),
            self.left.wait_for_rising_edge(),
            self.right.wait_for_rising_edge(),
        )
        .await;

        let rising = Instant::now();

        match button {
            Either4::First(_) => self.down.wait_for_falling_edge().await,
            Either4::Second(_) => self.up.wait_for_falling_edge().await,
            Either4::Third(_) => self.left.wait_for_falling_edge().await,
            Either4::Fourth(_) => self.right.wait_for_falling_edge().await,
        }
        .expect("unexpected error");

        let falling = Instant::now();
        let duration = falling - rising;

        let direction = match button {
            Either4::First(_) => Direction::Down,
            Either4::Second(_) => Direction::Up,
            Either4::Third(_) => Direction::Left,
            Either4::Fourth(_) => Direction::Right,
        };

        Input {
            duration,
            direction,
        }
    }
}

pub struct InputModule {
    _priv: (),
}

impl BusModule for InputModule {
    type Params = InputPins;
    type Event = Input;

    fn init(
        event_bus: &'static events::Bus<Self::Event>,
        params: Self::Params,
    ) -> crate::module::Spawnable<WithBus<Self>, impl Sized> {
        let token = input_task(event_bus, params);
        Spawnable::new_by_token(WithBus::new(event_bus, Self { _priv: () }), token)
    }
}

#[embassy_executor::task]
pub async fn input_task(event_bus: &'static events::Bus<Input>, mut pins: InputPins) {
    loop {
        let input = pins.wait_for_any().await;
        event_bus.send(input).await;
    }
}
