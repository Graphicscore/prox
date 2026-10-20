mod commands;
mod creds;

use std::sync::Arc;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(commands::App {
            client: proxer_client::Client::new("https://proxer.me"),
            store: Arc::new(creds::KeyringStore),
        })
        .invoke_handler(tauri::generate_handler![
            commands::login, commands::auto_login, commands::logout,
            commands::search, commands::get_info, commands::get_episodes, commands::get_streams, commands::resolve_stream
        ])
        .run(tauri::generate_context!())
        .expect("tauri run");
}
