use esp32c3_hal::peripherals::WIFI;
use esp_wifi::{
    esp_now::{EspNow, ReceiveInfo},
    EspWifiInitialization,
};
use heapless::String;
use serde::{Deserialize, Serialize};

use crate::{
    events::Bus,
    module::{BusModule, Spawnable},
};

#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    Text(String<16>),
}

#[derive(Debug)]
pub struct NetworkEvent {
    receive_info: ReceiveInfo,
    message: NetworkMessage,
}

#[embassy_executor::task]
pub async fn network_task(
    wifi: WIFI,
    token: EspWifiInitialization,
    event_bus: &'static Bus<NetworkEvent>,
) {
    let mut espnow = EspNow::new(&token, wifi).unwrap();

    // espnow.(&[0xFF; 6], b"ciao");
    // espnow.send_async(&[0xFF; 6], b"ciao").await.unwrap();

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

pub struct NetworkDevice {}

impl BusModule for NetworkDevice {
    type Params = (WIFI, EspWifiInitialization);
    type Event = NetworkEvent;

    fn init(
        event_bus: &'static Bus<Self::Event>,
        (wifi, token): Self::Params,
    ) -> Spawnable<Self, impl Sized> {
        let task = network_task(wifi, token, event_bus);
        Spawnable::new_by_token(Self {}, task)
    }
}
