use anyhow::{anyhow, Result};
use rumqttc::MqttOptions;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum TlsOptions {
    NativePem(Vec<u8>),
    NativeDer(Vec<u8>),
}

pub(crate) fn mqtt_options_from_uri(uri: &str) -> Result<MqttOptions> {
    let parsed_url = Url::parse(uri)?;
    let scheme = parsed_url.scheme();
    let use_tls = match scheme {
        "mqtts" => true,
        _ => false,
    };
    let default_port = if use_tls { 8883 } else { 1883 };
    let host = parsed_url.host_str().ok_or(anyhow!("Invalid host"))?;
    let port = parsed_url.port().unwrap_or(default_port);

    let client_id = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "client_id")
        .map(|(_, value)| value.to_string())
        .unwrap_or_else(|| "tauri-plugin-mqtt-client".to_string());
    let mut options = MqttOptions::new(client_id, host, port);

    let username = parsed_url.username();
    let password = parsed_url.password();
    if !username.is_empty() {
        options.set_credentials(username, password.unwrap_or(""));
    }


    if let Some(keep_alive) = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "keep_alive")
        .map(|(_, value)| value.parse::<u64>().unwrap_or(5))
    {
        options.set_keep_alive(Duration::from_secs(keep_alive));
    }
    if let Some(clean_session) = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "clean_session")
        .map(|(_, value)| value.parse::<bool>().unwrap_or(true))
    {
        options.set_clean_session(clean_session);
    }
    let incoming = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "max_incoming_packet_size")
        .map(|(_, value)| value.parse::<usize>().unwrap_or(10240))
        .unwrap_or(10240usize);
    let outgoing = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "max_outgoing_packet_size")
        .map(|(_, value)| value.parse::<usize>().unwrap_or(10240))
        .unwrap_or(10240usize);
    options.set_max_packet_size(incoming, outgoing);

    Ok(options)
}
