use core::borrow::Borrow;

use alloc::boxed::Box;
use esp32c3_hal::peripherals::WIFI;
use esp_wifi::{
    esp_now::{EspNow, EspNowManager, EspNowReceiver, EspNowSender, ReceiveInfo},
    EspWifiInitialization,
};
use heapless::String;
use serde::{Deserialize, Serialize};

use crate::{
    events::Bus,
    module::{BusModule, Spawnable, WithBus},
};

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    Text(String<16>),
}

#[derive(Debug)]
pub struct NetworkEvent {
    pub receive_info: ReceiveInfo,
    pub message: NetworkMessage,
}

#[embassy_executor::task]
pub async fn network_task(
    mut espnow: EspNowReceiver<'static>,
    event_bus: &'static Bus<NetworkEvent>,
) {
    loop {
        let received = espnow.receive_async().await;

        let Ok(message) =
            postcard::from_bytes::<NetworkMessage>(&received.data[..received.len as usize])
        else {
            log::error!("Received a weird packet! EWWWWW");
            continue;
        };

        let event = NetworkEvent {
            receive_info: received.info,
            message,
        };

        event_bus.send(event).await;
        log::info!("{received:?}");
    }
}

pub struct NetworkModule {
    manager: EspNowManager<'static>,
    sender: EspNowSender<'static>,

    buffer: Box<[u8; 256]>,
}

impl NetworkModule {
    const BROADCAST: &'static [u8; 6] = &[0xFF; 6];

    pub async fn send_message(&mut self, message: impl Borrow<NetworkMessage>) {
        let message = message.borrow();
        let serialized = postcard::to_slice(message, &mut self.buffer[..]).unwrap();

        self.sender
            .send_async(Self::BROADCAST, &serialized)
            .await
            .unwrap();
    }
}

impl BusModule for NetworkModule {
    type Params = (WIFI, EspWifiInitialization);
    type Event = NetworkEvent;

    fn init(
        event_bus: &'static Bus<Self::Event>,
        (wifi, token): Self::Params,
    ) -> Spawnable<WithBus<Self>, impl Sized> {
        let espnow = EspNow::new(&token, wifi).unwrap();
        let (manager, sender, receiver) = espnow.split();

        let task = network_task(receiver, event_bus);

        let module = Self {
            manager,
            sender,
            buffer: Box::new([0; 256]),
        };

        Spawnable::new_by_token(WithBus::new(event_bus, module), task)
    }
}
