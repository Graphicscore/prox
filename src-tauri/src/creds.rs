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
    /// Rejects a username containing '\n' (it would corrupt the `user\npass` format).
    fn save(&self, user: &str, pass: &str) -> Result<(), String> {
        if user.contains('\n') {
            return Err("username must not contain a newline".into());
        }
        use std::io::Write;
        let dir = self.path.parent().ok_or("no parent dir")?;
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        // Temp file in the same dir, created 0600 from the start, then renamed over the target (atomic).
        let tmp = self.path.with_extension("tmp");
        let _ = std::fs::remove_file(&tmp);
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }
        let res = opts
            .open(&tmp)
            .and_then(|mut f| f.write_all(format!("{user}\n{pass}").as_bytes()).and_then(|_| f.sync_all()))
            .and_then(|_| std::fs::rename(&tmp, &self.path));
        if let Err(e) = res {
            let _ = std::fs::remove_file(&tmp);
            return Err(e.to_string());
        }
        Ok(())
    }
    fn load(&self) -> Option<(String, String)> {
        let s = std::fs::read_to_string(&self.path).ok()?;
        let (u, p) = s.split_once('\n')?;
        Some((u.into(), p.into()))
    }
    fn clear(&self) {
        // Errors ignored on purpose: the trait returns (), and a missing file is the normal case.
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

    #[test]
    fn file_store_empty_password() {
        let dir = temp_dir("empty");
        let s = FileStore::new(&dir);
        s.save("u", "").unwrap();
        assert_eq!(s.load(), Some(("u".into(), "".into())));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_store_rejects_newline_username() {
        let dir = temp_dir("nl");
        let s = FileStore::new(&dir);
        assert!(s.save("a\nb", "p").is_err());
        assert!(!dir.join("credentials").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn file_store_tightens_existing_loose_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = temp_dir("loose");
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("credentials");
        std::fs::write(&f, "old\nold").unwrap();
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o644)).unwrap();
        FileStore::new(&dir).save("u", "p").unwrap();
        assert_eq!(std::fs::metadata(&f).unwrap().permissions().mode() & 0o777, 0o600);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_store_leaves_no_temp_file() {
        let dir = temp_dir("tmp");
        let s = FileStore::new(&dir);
        s.save("u", "p").unwrap();
        s.save("u", "p2").unwrap();
        let names: Vec<_> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, vec![std::ffi::OsString::from("credentials")]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
