use tauri::Runtime;

use crate::{error::Result, mqtt_options::TlsOptions};

#[cfg(desktop)]
use crate::desktop as platform;

#[tauri::command]
pub async fn subscribe<R: Runtime>(
    _window: tauri::Window<R>,
    id: String,
    topic: String,
    qos: u8,
) -> Result<()> {
    platform::subscribe(id, topic, qos)
        .await
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn unsubscribe<R: Runtime>(
    _window: tauri::Window<R>,
    id: String,
    topic: String,
) -> Result<()> {
    platform::unsubscribe(id, topic)
        .await
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn publish<R: Runtime>(
    _window: tauri::Window<R>,
    id: String,
    topic: String,
    qos: u8,
    retain: bool,
    payload: Vec<u8>,
) -> Result<()> {
    platform::publish(id, topic, qos, retain, payload).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn connect<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    uri: String,
    tls_options: Option<TlsOptions>,
) -> Result<()> {
    platform::connect(window, id, uri, tls_options)
        .await
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn disconnect(id: String) -> Result<()> {
    platform::disconnect(id).await.map_err(|e| e.into())
}
