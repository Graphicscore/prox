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
}
