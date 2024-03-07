use embassy_executor::{SpawnToken, Spawner};

use crate::events::Bus;

pub struct Spawnable<T, S> {
    output: T,
    token: SpawnToken<S>,
}

impl<T, S> Spawnable<T, S> {
    pub fn spawn(self, spawner: &Spawner) -> T {
        spawner.spawn(self.token).unwrap();

        self.output
    }

    pub fn new_by_token(module: T, token: SpawnToken<S>) -> Self {
        Self {
            output: module,
            token,
        }
    }
}

pub trait BusModule: Sized {
    type Params;
    type Event;

    fn init(
        event_bus: &'static Bus<Self::Event>,
        params: Self::Params,
    ) -> Spawnable<Self, impl Sized>;
}
