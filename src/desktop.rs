use std::collections::HashMap;

use debug_print::debug_println;
use lazy_static::lazy_static;
use rumqttc::{
    tokio_rustls::rustls::{server, ClientConfig},
    AsyncClient, Event, Packet, QoS, TlsConfiguration, Transport,
};
use tauri::{Emitter, Manager, Runtime};
use tokio::{
    io::{self},
    sync::RwLock,
    time::{self, sleep},
};
use tokio_native_tls::native_tls;

use crate::{
    models::*,
    mqtt_options::{mqtt_options_from_uri, SkipServerVerification, TlsOptions},
};

lazy_static! {
    static ref CLIENTS: RwLock<HashMap<String, Mqtt>> = RwLock::new(HashMap::new());
}

pub(crate) async fn connect<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    uri: String,
    tls_options: Option<TlsOptions>,
) -> io::Result<()> {
    debug_println!("tls_options: {:?}", &tls_options);
    let mut clients = CLIENTS.write().await;

    if let Some(s) = clients.get(&id) {
        s.task.abort();
        clients.remove(&id);
        sleep(time::Duration::from_millis(100)).await;
    }

    let mut options = mqtt_options_from_uri(&uri).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Failed to parse uri {}", &uri),
        )
    })?;

    if let Some(tls_options) = tls_options {
        match tls_options {
            TlsOptions::SkipVerification(true) => {
                let config = ClientConfig::builder()
                    .dangerous()
                    .with_custom_certificate_verifier(SkipServerVerification::new())
                    .with_no_client_auth();
                options.set_transport(Transport::tls_with_config(config.into()));
            }
            TlsOptions::Simple { ca, alpn, client_key, client_cert }=> {
                let client_auth = match (client_key, client_cert) {
                    (Some(client_key), Some(client_cert)) => {
                        Some((client_key, client_cert))
                    },
                    _ => None,
                };
                let transport = Transport::Tls(TlsConfiguration::Simple {
                    ca,
                    alpn,
                    client_auth,
                });
            
                options.set_transport(transport);
            }
            _ => {}
        }
    }

    let (client, mut event_loop) = AsyncClient::new(options, 10);

    let mqtt_id = id.clone();
    let task = tokio::task::spawn(async move {
        loop {
            match event_loop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(c))) => {
                    debug_println!("Connected to broker");
                    let _ = window.app_handle().emit_to(
                        window.label(),
                        "plugin://mqtt",
                        Payload {
                            id: mqtt_id.clone(),
                            event: PayloadEvent::Connect(),
                        },
                    );
                }
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
                    // 不能只移除，会导致无法手动disconnect
                    //CLIENTS.write().await.remove(&mqtt_id);
                    let _ = window.app_handle().emit_to(
                        window.label(),
                        "plugin://mqtt",
                        Payload {
                            id: mqtt_id.clone(),
                            event: PayloadEvent::Disconnect(),
                        },
                    );
                    // 发生错误停止进程并移除
                    if let Some(s) = clients.get(&mqtt_id) {
                        s.task.abort();
                        CLIENTS.write().await.remove(&mqtt_id);
                        sleep(time::Duration::from_millis(100)).await;
                    }
                    break;
                }
                e => {
                    debug_println!("e = {e:?}");
                }
            }
        }
    });

    clients.insert(id, Mqtt { task, client });
    Ok(())
}

pub(crate) async fn disconnect(id: String) -> io::Result<()> {
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

pub(crate) async fn subscribe(id: String, topic: String, qos: u8) -> io::Result<()> {
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

pub(crate) async fn unsubscribe(id: String, topic: String) -> io::Result<()> {
    let clients = CLIENTS.read().await;

    if let Some(s) = clients.get(&id) {
        s.client.unsubscribe(&topic).await.map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to unsubscribe to topic {}", topic),
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

pub(crate) async fn publish(
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
