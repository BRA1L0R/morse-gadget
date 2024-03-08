use core::ops::{Deref, DerefMut};

use embassy_executor::{SpawnToken, Spawner};

use crate::events::Bus;

/// A task that can be used only when spawned
/// onto an embassy executor
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

/// Defines a module that has been initialized
/// along-side a bus. Implements DerefMut so it's transparent
/// to the module and can be spliced into it
pub struct WithBus<T: BusModule> {
    bus: &'static Bus<T::Event>,
    obj: T,
}

impl<T> WithBus<T>
where
    T: BusModule,
{
    pub fn new(bus: &'static Bus<T::Event>, obj: T) -> Self {
        Self { bus, obj }
    }

    pub async fn receive_event(&self) -> T::Event {
        self.bus.receive().await
    }

    pub fn into_inner(self) -> (T, &'static Bus<T::Event>) {
        let Self { obj, bus } = self;
        (obj, bus)
    }
}

impl<T: BusModule> Deref for WithBus<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.obj
    }
}

impl<T: BusModule> DerefMut for WithBus<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.obj
    }
}

pub trait BusModule: Sized {
    type Params;
    type Event: 'static;

    fn init(
        event_bus: &'static Bus<Self::Event>,
        params: Self::Params,
    ) -> Spawnable<WithBus<Self>, impl Sized>;
}
