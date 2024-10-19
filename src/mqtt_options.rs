use anyhow::{anyhow, Result};
use rumqttc::{tokio_rustls::rustls::client::danger::ServerCertVerifier, MqttOptions};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) enum TlsOptions {
    SkipVerification(bool),
    #[serde(untagged)]
    Simple {
        ca: Vec<u8>,
        alpn: Option<Vec<Vec<u8>>>,
        client_cert: Option<Vec<u8>>, 
        client_key: Option<Vec<u8>>, 
    },
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

#[derive(Debug)]
pub(crate) struct SkipServerVerification;

impl SkipServerVerification {
    pub fn new() -> std::sync::Arc<Self> {
        std::sync::Arc::new(Self)
    }
}

impl ServerCertVerifier for SkipServerVerification {
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rumqttc::tokio_rustls::rustls::pki_types::CertificateDer<'_>,
        _dss: &rumqttc::tokio_rustls::rustls::DigitallySignedStruct,
    ) -> std::result::Result<
        rumqttc::tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
        rumqttc::tokio_rustls::rustls::Error,
    > {
        Ok(rumqttc::tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rumqttc::tokio_rustls::rustls::pki_types::CertificateDer<'_>,
        _dss: &rumqttc::tokio_rustls::rustls::DigitallySignedStruct,
    ) -> std::result::Result<
        rumqttc::tokio_rustls::rustls::client::danger::HandshakeSignatureValid,
        rumqttc::tokio_rustls::rustls::Error,
    > {
        Ok(rumqttc::tokio_rustls::rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rumqttc::tokio_rustls::rustls::SignatureScheme> {
        vec![
            rumqttc::tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rumqttc::tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rumqttc::tokio_rustls::rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rumqttc::tokio_rustls::rustls::SignatureScheme::ECDSA_SHA1_Legacy,
            rumqttc::tokio_rustls::rustls::SignatureScheme::ED25519,
            rumqttc::tokio_rustls::rustls::SignatureScheme::ED448,
            rumqttc::tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA1,
            rumqttc::tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rumqttc::tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rumqttc::tokio_rustls::rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rumqttc::tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA256,
            rumqttc::tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA384,
            rumqttc::tokio_rustls::rustls::SignatureScheme::RSA_PSS_SHA512,
        ]
    }

    fn verify_server_cert(
        &self,
        _end_entity: &rumqttc::tokio_rustls::rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rumqttc::tokio_rustls::rustls::pki_types::CertificateDer<'_>],
        _server_name: &rumqttc::tokio_rustls::rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rumqttc::tokio_rustls::rustls::pki_types::UnixTime,
    ) -> std::result::Result<
        rumqttc::tokio_rustls::rustls::client::danger::ServerCertVerified,
        rumqttc::tokio_rustls::rustls::Error,
    > {
        Ok(rumqttc::tokio_rustls::rustls::client::danger::ServerCertVerified::assertion())
    }
}
