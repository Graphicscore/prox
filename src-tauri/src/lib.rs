mod commands;
mod creds;

use std::sync::Arc;
use tauri::Manager;

#[cfg(target_os = "android")]
fn make_store(app: &tauri::App) -> Result<Arc<dyn creds::CredStore>, Box<dyn std::error::Error>> {
    Ok(Arc::new(creds::FileStore::new(&app.path().app_data_dir()?)))
}

#[cfg(not(target_os = "android"))]
fn make_store(_app: &tauri::App) -> Result<Arc<dyn creds::CredStore>, Box<dyn std::error::Error>> {
    Ok(Arc::new(creds::KeyringStore))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(commands::App { client: proxer_client::Client::new("https://proxer.me"), store: make_store(app)? });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::login, commands::auto_login, commands::logout,
            commands::search, commands::get_list, commands::get_info, commands::get_episodes, commands::get_streams, commands::resolve_stream
        ])
        .run(tauri::generate_context!())
        .expect("tauri run");
}
