use iced::{
    Element, Subscription,
    futures::{SinkExt, StreamExt, channel::mpsc::Sender, select, stream::Fuse},
    stream,
};
use zbus::{Connection, proxy, proxy::PropertyStream, zvariant::OwnedObjectPath};

use crate::{bar::BAR_WIDTH, icon};

#[derive(Debug)]
pub struct WiFi(Option<u8>);

#[derive(Clone, Debug)]
pub struct WiFiMessage(Option<u8>);

impl WiFi {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn update(&mut self, message: WiFiMessage) {
        self.0 = message.0;
    }

    pub fn view(&self, width: f32) -> Element<'_, WiFiMessage> {
        let size = 18.0 / BAR_WIDTH * width;

        match self.0 {
            None => icon::wifi_off().size(size).line_height(1.0).into(),
            Some(0..25) => icon::wifi_1().size(size).line_height(1.0).into(),
            Some(25..50) => icon::wifi_2().size(size).line_height(1.0).into(),
            Some(50..75) => icon::wifi_3().size(size).line_height(1.0).into(),
            Some(75..) => icon::wifi_4().size(size).line_height(1.0).into(),
        }
    }

    pub fn subscription(&self) -> Subscription<WiFiMessage> {
        Subscription::run(|| stream::channel(64, Self::wifi))
    }

    async fn wifi(mut sender: Sender<WiFiMessage>) {
        let connection = Connection::system().await.unwrap();
        let network_manager = NetworkManagerProxy::new(&connection).await.unwrap();

        let mut primary_connection_stream = network_manager
            .receive_primary_connection_changed()
            .await
            .fuse();
        let mut current_strength_stream = None;

        loop {
            let Some(strength_stream) = &mut current_strength_stream else {
                let property = primary_connection_stream.next().await.unwrap();
                Self::update_strength_stream(
                    property.get().await.unwrap(),
                    &mut current_strength_stream,
                    &connection,
                    &mut sender,
                )
                .await;
                continue;
            };

            select! {
                x = primary_connection_stream.next() => Self::update_strength_stream(
                    x.unwrap().get().await.unwrap(),
                    &mut current_strength_stream,
                    &connection,
                    &mut sender,
                )
                .await,
                x = strength_stream.next() => {
                    let value = Some(x.unwrap().get().await.unwrap());
                    sender.send(WiFiMessage(value)).await.unwrap();
                }
            }
        }
    }

    async fn update_strength_stream<'a>(
        path: OwnedObjectPath,
        stream: &mut Option<Fuse<PropertyStream<'a, u8>>>,
        connection: &'a Connection,
        sender: &mut Sender<WiFiMessage>,
    ) {
        if path.as_ref() == "/" {
            *stream = None;
            sender.send(WiFiMessage(None)).await.unwrap();
            return;
        }

        let active = ActiveConnectionProxy::builder(connection)
            .path(path)
            .unwrap()
            .build()
            .await
            .unwrap();
        let device = active.devices().await.unwrap().into_iter().next().unwrap();

        let wireless = WirelessDeviceProxy::builder(connection)
            .path(device)
            .unwrap()
            .build()
            .await
            .unwrap();

        let access_point_path = wireless.active_access_point().await.unwrap();
        let access_point = AccessPointProxy::builder(connection)
            .path(access_point_path)
            .unwrap()
            .build()
            .await
            .unwrap();

        *stream = Some(access_point.receive_strength_changed().await.fuse());
    }
}

#[proxy(interface = "org.freedesktop.NetworkManager", assume_defaults = true)]
trait NetworkManager {
    #[zbus(property)]
    fn primary_connection(&self) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    assume_defaults = true
)]
trait ActiveConnection {
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<OwnedObjectPath>>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.Device.Wireless",
    assume_defaults = true
)]
trait WirelessDevice {
    #[zbus(property)]
    fn active_access_point(&self) -> zbus::Result<OwnedObjectPath>;
}

#[proxy(
    interface = "org.freedesktop.NetworkManager.AccessPoint",
    assume_defaults = true
)]
trait AccessPoint {
    #[zbus(property)]
    fn strength(&self) -> zbus::Result<u8>;
}
