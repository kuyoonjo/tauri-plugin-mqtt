use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;
pub mod error;
pub mod models;
pub mod mqtt_options;

#[cfg(desktop)]
pub mod desktop;

const PLUGIN_NAME: &str = "mqtt";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::subscribe,
            commands::publish,
        ])
        .build()
}
