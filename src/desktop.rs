use std::collections::HashMap;

use debug_print::debug_println;
use lazy_static::lazy_static;
use rumqttc::{AsyncClient, Event, Packet, QoS};
use tauri::{Emitter, Manager, Runtime};
use tokio::{
    io::{self},
    sync::RwLock,
    time::{self, sleep},
};

use crate::{models::*, mqtt_options::mqtt_options_from_uri};

lazy_static! {
    static ref CLIENTS: RwLock<HashMap<String, Mqtt>> = RwLock::new(HashMap::new());
}

pub async fn connect<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    uri: String,
) -> io::Result<()> {
    let mut clients = CLIENTS.write().await;

    if let Some(s) = clients.get(&id) {
        s.task.abort();
        clients.remove(&id);
        sleep(time::Duration::from_millis(100)).await;
    }

    let option = mqtt_options_from_uri(&uri).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Failed to parse uri {}", &uri),
        )
    })?;
    let (client, mut event_loop) = AsyncClient::new(option, 10);

    let _ = window.app_handle().emit_to(
        window.label(),
        "plugin://mqtt",
        Payload {
            id: id.clone(),
            event: PayloadEvent::Connect(),
        },
    );

    let mqtt_id = id.clone();
    let task = tokio::task::spawn(async move {
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    debug_println!("Topic: {}, Payload: {:?}", p.topic, p.payload);
                    let message = MqttPublish::from(p);
                    let _ = window.app_handle().emit_to(
                        window.label(),
                        "plugin://mqtt",
                        Payload {
                            id: mqtt_id.clone(),
                            event: PayloadEvent::Message(message),
                        },
                    );
                }
                Err(e) => {
                    debug_println!("Error = {e:?}");
                    let _ = window.app_handle().emit_to(
                        window.label(),
                        "plugin://mqtt",
                        Payload {
                            id: mqtt_id.clone(),
                            event: PayloadEvent::Disconnect(),
                        },
                    );
                }
                _ => {}
            }
        }
    });

    clients.insert(id, Mqtt { task, client });
    Ok(())
}

pub async fn disconnect(id: String) -> io::Result<()> {
    let mut clients = CLIENTS.write().await;

    if let Some(s) = clients.get(&id) {
        s.task.abort();
        clients.remove(&id);
        debug_println!("{} mqtt disconnected", &id);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("ID {} not disconnected.", &id),
        ))
    }
}

pub async fn subscribe(id: String, topic: String, qos: u8) -> io::Result<()> {
    let clients = CLIENTS.read().await;

    if let Some(s) = clients.get(&id) {
        let qos = match qos {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid QoS")),
        };
        s.client
            .subscribe(&topic, QoS::from(qos))
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to subscribe to topic {}", topic),
                )
            })?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("ID {} not connected.", &id),
        ))
    }
}

pub async fn publish(
    id: String,
    topic: String,
    qos: u8,
    retain: bool,
    payload: Vec<u8>,
) -> io::Result<()> {
    let clients = CLIENTS.read().await;

    if let Some(s) = clients.get(&id) {
        let qos = match qos {
            0 => QoS::AtMostOnce,
            1 => QoS::AtLeastOnce,
            2 => QoS::ExactlyOnce,
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid QoS")),
        };
        s.client
            .publish(&topic, QoS::from(qos), retain, payload)
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to publish to topic {}", topic),
                )
            })?;
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("ID {} not connected.", &id),
        ))
    }
}
