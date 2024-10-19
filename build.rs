const COMMANDS: &[&str] = &[
    "connect",
    "disconnect",
    "publish",
    "subscribe",
    "unsubscribe",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
