use iced::futures::{SinkExt, StreamExt, channel::mpsc::Sender, select, stream::Fuse};
use rusty_network_manager::{AccessPointProxy, ActiveProxy, NetworkManagerProxy, WirelessProxy};
use zbus::{Connection, proxy::PropertyStream, zvariant::OwnedObjectPath};

use crate::shell::Message;

pub async fn wifi(mut sender: Sender<Message>) {
    let connection = Connection::system().await.unwrap();
    let network_manager = NetworkManagerProxy::new(&connection).await.unwrap();

    let primary_path = network_manager.primary_connection().await.unwrap();
    let primary = ActiveProxy::new_from_path(primary_path, &connection)
        .await
        .unwrap();
    let device = primary.devices().await.unwrap().into_iter().next().unwrap();

    let wireless = WirelessProxy::new_from_path(device, &connection)
        .await
        .unwrap();

    let mut access_point_stream = wireless.receive_active_access_point_changed().await.fuse();
    let mut current_strength_stream = None;

    loop {
        let strength_stream = match &mut current_strength_stream {
            None => {
                let property = access_point_stream.next().await.unwrap();
                update_strength_stream(
                    property.get().await.unwrap(),
                    &mut current_strength_stream,
                    &connection,
                    &mut sender,
                )
                .await;
                match &mut current_strength_stream {
                    None => continue,
                    Some(x) => x,
                }
            }
            Some(x) => x,
        };

        select! {
            x = access_point_stream.next() => update_strength_stream(
                x.unwrap().get().await.unwrap(),
                &mut current_strength_stream,
                &connection,
                &mut sender,
            )
            .await,
            x = strength_stream.next() => {
                let value = Some(x.unwrap().get().await.unwrap());
                sender
                    .send(Message::WifiStrengthChanged(value))
                    .await
                    .unwrap();
            }
        }
    }
}

async fn update_strength_stream<'a>(
    path: OwnedObjectPath,
    stream: &mut Option<Fuse<PropertyStream<'a, u8>>>,
    connection: &'a Connection,
    sender: &mut Sender<Message>,
) {
    if path.as_ref() == "/" {
        *stream = None;
        sender
            .send(Message::WifiStrengthChanged(None))
            .await
            .unwrap();
        return;
    }

    let access_point = AccessPointProxy::new_from_path(path, connection)
        .await
        .unwrap();
    *stream = Some(access_point.receive_strength_changed().await.fuse());
}
