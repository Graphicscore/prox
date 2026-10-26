//! Self-update from the public Forgejo releases of asteria/prox on git.asteria.dog (CI publishes `build-<N>` with
//! fixed asset names from a `build-<N>` tag; the app's own version is `0.1.<N>`). Android hands the APK to the system installer through the
//! `AndroidTv.installUpdate` bridge (MainActivity.kt); a Linux AppImage replaces itself and restarts.
use serde::Serialize;
use tauri::AppHandle;

const LATEST: &str = "https://git.asteria.dog/api/v1/repos/asteria/prox/releases/latest";
const DOWNLOAD_PREFIX: &str = "https://git.asteria.dog/asteria/prox/releases/download/";
const RELEASES: &str = "https://git.asteria.dog/asteria/prox/releases";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Update {
    pub build: u32,
    pub current: u32,
    /// Download for this device; None = no self-install here (deb, dev build, unsupported ABI): show `page`.
    pub url: Option<String>,
    pub size: u64,
    pub page: String,
}

pub fn tag_build(tag: &str) -> Option<u32> {
    tag.strip_prefix("build-")?.parse().ok()
}

/// "0.1.42" -> 42; local builds ("0.1.0") are build 0, so any release counts as newer.
pub fn version_build(v: &str) -> u32 {
    v.rsplit('.').next().and_then(|p| p.parse().ok()).unwrap_or(0)
}

pub fn asset_name(os: &str, arch: &str, appimage: bool) -> Option<&'static str> {
    match (os, arch) {
        ("android", "aarch64") => Some("proxer-android-arm64.apk"),
        ("android", "arm") => Some("proxer-android-armv7.apk"),
        ("android", "x86_64") => Some("proxer-android-x86_64.apk"),
        ("linux", "x86_64") if appimage => Some("proxer-linux-x86_64.AppImage"),
        _ => None,
    }
}

/// The latest-release JSON: build number, release page and (url, size) of the asset called `name`.
/// Only URLs below our own release downloads are accepted.
pub fn parse_release(json: &str, name: Option<&str>) -> Option<(u32, String, Option<(String, u64)>)> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let build = tag_build(v.get("tag_name")?.as_str()?)?;
    let page = v.get("html_url").and_then(|u| u.as_str()).filter(|u| u.starts_with(RELEASES)).unwrap_or(RELEASES).to_string();
    let asset = name.and_then(|n| {
        let a = v.get("assets")?.as_array()?.iter().find(|a| a.get("name").and_then(|x| x.as_str()) == Some(n))?;
        let url = a.get("browser_download_url")?.as_str()?;
        url.starts_with(DOWNLOAD_PREFIX).then(|| (url.to_string(), a.get("size").and_then(|s| s.as_u64()).unwrap_or(0)))
    });
    Some((build, page, asset))
}

fn http() -> reqwest::Client {
    proxer_client::with_tls(reqwest::Client::builder())
        .user_agent("proxer-app-updater")
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .expect("http client")
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<Option<Update>, String> {
    let current = version_build(&app.package_info().version.to_string());
    let name = asset_name(std::env::consts::OS, std::env::consts::ARCH, std::env::var_os("APPIMAGE").is_some());
    let res = http().get(LATEST).header("Accept", "application/json").send().await.map_err(|_| "Network".to_string())?;
    if res.status().as_u16() == 404 {
        return Ok(None); // no release published yet
    }
    if !res.status().is_success() {
        return Err(format!("Http:{}", res.status().as_u16()));
    }
    let body = res.text().await.map_err(|_| "Network".to_string())?;
    let (build, page, asset) = parse_release(&body, name).ok_or_else(|| "Parse:release".to_string())?;
    if build <= current {
        return Ok(None);
    }
    let (url, size) = match asset { Some((u, s)) => (Some(u), s), None => (None, 0) };
    Ok(Some(Update { build, current, url, size, page }))
}

/// Download one of our release assets. Android: returns the APK path for `AndroidTv.installUpdate`.
/// Linux AppImage: replaces the running AppImage, starts the new one and exits.
#[tauri::command]
pub async fn download_update(app: AppHandle, url: String) -> Result<String, String> {
    if !url.starts_with(DOWNLOAD_PREFIX) {
        return Err("Unsupported".into());
    }
    let res = http().get(&url).send().await.map_err(|_| "Network".to_string())?;
    if !res.status().is_success() {
        return Err(format!("Http:{}", res.status().as_u16()));
    }
    let bytes = res.bytes().await.map_err(|_| "Network".to_string())?;
    install(&app, &bytes)
}

#[cfg(target_os = "android")]
fn install(app: &AppHandle, bytes: &[u8]) -> Result<String, String> {
    use tauri::Manager;
    let dir = app.path().app_cache_dir().map_err(|_| "Unsupported".to_string())?.join("updates");
    std::fs::create_dir_all(&dir).map_err(|_| "Io".to_string())?;
    let path = dir.join("update.apk");
    std::fs::write(&path, bytes).map_err(|_| "Io".to_string())?;
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(not(target_os = "android"))]
fn install(app: &AppHandle, bytes: &[u8]) -> Result<String, String> {
    use std::os::unix::fs::PermissionsExt;
    let target = std::path::PathBuf::from(std::env::var_os("APPIMAGE").ok_or_else(|| "Unsupported".to_string())?);
    let tmp = target.with_extension("update");
    std::fs::write(&tmp, bytes).map_err(|_| "Io".to_string())?;
    std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)).map_err(|_| "Io".to_string())?;
    std::fs::rename(&tmp, &target).map_err(|_| "Io".to_string())?; // same directory: atomic swap
    std::process::Command::new(&target).spawn().map_err(|_| "Io".to_string())?;
    app.exit(0);
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REL: &str = r#"{"tag_name":"build-42","html_url":"https://git.asteria.dog/asteria/prox/releases/tag/build-42","assets":[
        {"name":"proxer-android-arm64.apk","size":18000000,"browser_download_url":"https://git.asteria.dog/asteria/prox/releases/download/build-42/proxer-android-arm64.apk"},
        {"name":"proxer-android-armv7.apk","size":17000000,"browser_download_url":"https://evil.example/x.apk"},
        {"name":"proxer-android-x86_64.apk","size":19000000,"browser_download_url":"https://github.com/Graphicscore/prox/releases/download/build-42/proxer-android-x86_64.apk"}]}"#;

    #[test]
    fn builds() {
        assert_eq!(tag_build("build-42"), Some(42));
        assert_eq!(tag_build("v1.0"), None);
        assert_eq!(version_build("0.1.42"), 42);
        assert_eq!(version_build("0.1.0"), 0);
        assert_eq!(version_build("weird"), 0);
    }

    #[test]
    fn assets_per_device() {
        assert_eq!(asset_name("android", "aarch64", false), Some("proxer-android-arm64.apk"));
        assert_eq!(asset_name("android", "arm", false), Some("proxer-android-armv7.apk"));
        assert_eq!(asset_name("android", "x86", false), None);
        assert_eq!(asset_name("linux", "x86_64", true), Some("proxer-linux-x86_64.AppImage"));
        assert_eq!(asset_name("linux", "x86_64", false), None); // deb or dev build: no self-install
    }

    #[test]
    fn release_json() {
        let (b, page, a) = parse_release(REL, Some("proxer-android-arm64.apk")).unwrap();
        assert_eq!((b, page.as_str()), (42, "https://git.asteria.dog/asteria/prox/releases/tag/build-42"));
        assert_eq!(a, Some(("https://git.asteria.dog/asteria/prox/releases/download/build-42/proxer-android-arm64.apk".into(), 18000000)));
        assert_eq!(parse_release(REL, Some("proxer-android-armv7.apk")).unwrap().2, None); // foreign host refused
        assert_eq!(parse_release(REL, Some("proxer-android-x86_64.apk")).unwrap().2, None); // old GitHub mirror refused
        assert_eq!(parse_release(REL, Some("missing")).unwrap().2, None);
        assert_eq!(parse_release(REL, None).unwrap().2, None);
        assert!(parse_release("{}", None).is_none());
    }
}
