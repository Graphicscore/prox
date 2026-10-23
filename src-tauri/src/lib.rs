mod commands;
mod creds;
mod update;

use std::sync::Arc;
use tauri::Manager;

#[cfg(target_os = "android")]
fn init_platform_tls() {
    // reqwest 0.13's `rustls` feature verifies certs via rustls-platform-verifier,
    // which needs the JVM app context before the first HTTPS request (it panics otherwise).
    // Tauri's Android runtime publishes its context through ndk-context, so wire the two up.
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.expect("android JavaVM");
    let mut env = vm.attach_current_thread().expect("attach jni thread");
    let context = unsafe { jni::objects::JObject::from_raw(ctx.context().cast()) };
    let _ = rustls_platform_verifier::android::init_with_env(&mut env, context);
}

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
    #[cfg(target_os = "android")]
    init_platform_tls();

    tauri::Builder::default()
        .setup(|app| {
            app.manage(commands::App { client: proxer_client::Client::new("https://proxer.me"), store: make_store(app)? });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::login, commands::auto_login, commands::logout,
            commands::search, commands::get_list, commands::get_calendar, commands::get_info, commands::get_episodes, commands::get_streams, commands::resolve_stream,
            commands::my_list, commands::title_action, commands::reminder, commands::mark_watched,
            update::check_update, update::download_update
        ])
        .run(tauri::generate_context!())
        .expect("tauri run");
}
