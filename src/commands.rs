use tauri::Runtime;

use crate::error::Result;

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
    endpoint: String,
) -> Result<()> {
    platform::connect(window, id, endpoint)
        .await
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn disconnect(id: String) -> Result<()> {
    platform::disconnect(id).await.map_err(|e| e.into())
}
