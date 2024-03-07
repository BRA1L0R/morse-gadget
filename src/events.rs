use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};

use crate::{input::Input, network::NetworkEvent};

pub type Bus<T, const N: usize = 10> = Channel<CriticalSectionRawMutex, T, N>;

// #[derive(Debug)]
// pub enum Event {
//     Input(Input),
//     Network(NetworkEvent),
// }

// pub type Bus = Channel<CriticalSectionRawMutex, Event, 10>;
// static EVENTS: Bus = Channel::new();

// pub fn bus() -> &'static Bus {
//     &EVENTS
// }

// pub async fn listen_event() -> Event {
//     EVENTS.receive().await
// }
