pub trait CredStore: Send + Sync {
    fn save(&self, user: &str, pass: &str) -> Result<(), String>;
    fn load(&self) -> Option<(String, String)>;
    fn clear(&self);
}

pub struct KeyringStore;
const SERVICE: &str = "me.louven.proxer";

impl CredStore for KeyringStore {
    fn save(&self, user: &str, pass: &str) -> Result<(), String> {
        keyring::Entry::new(SERVICE, "account").and_then(|e| e.set_password(&format!("{user}\n{pass}"))).map_err(|e| e.to_string())
    }
    fn load(&self) -> Option<(String, String)> {
        let s = keyring::Entry::new(SERVICE, "account").ok()?.get_password().ok()?;
        let (u, p) = s.split_once('\n')?;
        Some((u.into(), p.into()))
    }
    fn clear(&self) {
        if let Ok(e) = keyring::Entry::new(SERVICE, "account") { let _ = e.delete_credential(); }
    }
}

/// App-private credential file for Android, where the `keyring` crate has no backend.
/// ponytail: plain text in the app's private directory (only this app can read it on a non-rooted device);
/// upgrade to the Android Keystore (Kotlin plugin) if the threat model needs it.
#[cfg_attr(not(target_os = "android"), allow(dead_code))]
pub struct FileStore {
    path: std::path::PathBuf,
}

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
impl FileStore {
    pub fn new(dir: &std::path::Path) -> Self {
        Self { path: dir.join("credentials") }
    }
}

impl CredStore for FileStore {
    fn save(&self, user: &str, pass: &str) -> Result<(), String> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(&self.path, format!("{user}\n{pass}")).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(0o600)).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    fn load(&self) -> Option<(String, String)> {
        let s = std::fs::read_to_string(&self.path).ok()?;
        let (u, p) = s.split_once('\n')?;
        Some((u.into(), p.into()))
    }
    fn clear(&self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
pub struct MemStore(pub std::sync::Mutex<Option<(String, String)>>);
#[cfg(test)]
impl CredStore for MemStore {
    fn save(&self, u: &str, p: &str) -> Result<(), String> { *self.0.lock().unwrap() = Some((u.into(), p.into())); Ok(()) }
    fn load(&self) -> Option<(String, String)> { self.0.lock().unwrap().clone() }
    fn clear(&self) { *self.0.lock().unwrap() = None; }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let s = MemStore(Default::default());
        assert!(s.load().is_none());
        s.save("u", "p").unwrap();
        assert_eq!(s.load(), Some(("u".into(), "p".into())));
        s.clear();
        assert!(s.load().is_none());
    }

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("proxer-creds-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn file_store_round_trip_and_mode() {
        let dir = temp_dir("rt");
        let s = FileStore::new(&dir);
        assert!(s.load().is_none());
        s.save("user", "pa\nss").unwrap();
        assert_eq!(s.load(), Some(("user".into(), "pa\nss".into())));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(dir.join("credentials")).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o600);
        }
        s.clear();
        assert!(s.load().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_store_malformed_file_is_none() {
        let dir = temp_dir("bad");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("credentials"), "no newline here").unwrap();
        assert!(FileStore::new(&dir).load().is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
