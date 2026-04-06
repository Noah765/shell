use iced::futures::{SinkExt, StreamExt, channel::mpsc::Sender, select, stream::Fuse};
use rusty_network_manager::{AccessPointProxy, ActiveProxy, NetworkManagerProxy, WirelessProxy};
use zbus::{Connection, proxy::PropertyStream, zvariant::OwnedObjectPath};

use crate::shell::Message;

pub async fn wifi(mut sender: Sender<Message>) {
    let connection = Connection::system().await.unwrap();
    let network_manager = NetworkManagerProxy::new(&connection).await.unwrap();

    let mut primary_connection_stream = network_manager
        .receive_primary_connection_changed()
        .await
        .fuse();
    let mut current_strength_stream = None;

    loop {
        let strength_stream = match &mut current_strength_stream {
            None => {
                let property = primary_connection_stream.next().await.unwrap();
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
            x = primary_connection_stream.next() => update_strength_stream(
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

    let active = ActiveProxy::new_from_path(path, connection).await.unwrap();
    let device = active.devices().await.unwrap().into_iter().next().unwrap();

    let wireless = WirelessProxy::new_from_path(device, connection)
        .await
        .unwrap();

    let access_point_path = wireless.active_access_point().await.unwrap();
    let access_point = AccessPointProxy::new_from_path(access_point_path, connection)
        .await
        .unwrap();

    *stream = Some(access_point.receive_strength_changed().await.fuse());
}
