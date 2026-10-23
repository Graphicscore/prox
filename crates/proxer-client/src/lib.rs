use tokio::sync::{Mutex, Semaphore};

const UA: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("not logged in")]
    NotLoggedIn,
    #[error("bad credentials")]
    BadCredentials,
    #[error("page structure unexpected: {0}")]
    Parse(&'static str),
    #[error("http status {0}")]
    Http(u16),
    #[error("network: {0}")]
    Network(#[from] reqwest::Error),
}

/// reqwest 0.13's `rustls` feature verifies against the platform certificate store via
/// rustls-platform-verifier. On Android that needs JVM glue (an AAR component) that a plain
/// Tauri app doesn't have, so every HTTPS request fails there. Use the static Mozilla root
/// bundle instead — this matches what reqwest 0.12's `rustls-tls` did and works everywhere.
#[cfg(target_os = "android")]
pub fn with_tls(b: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    use rustls::RootCertStore;
    let mut roots = RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let cfg = rustls::ClientConfig::builder_with_provider(rustls::crypto::aws_lc_rs::default_provider().into())
        .with_safe_default_protocol_versions().expect("safe protocol versions")
        .with_root_certificates(roots)
        .with_no_client_auth();
    b.tls_backend_preconfigured(cfg)
}

#[cfg(not(target_os = "android"))]
pub fn with_tls(b: reqwest::ClientBuilder) -> reqwest::ClientBuilder {
    b
}

pub struct Client {
    http: std::sync::RwLock<reqwest::Client>,
    embed_http: reqwest::Client, // anonymous: no cookies, no redirects
    base: String,
    creds: Mutex<Option<(String, String)>>,
    relogin: Mutex<()>, // single-flight re-login; taken before any semaphore permit
    gate: Semaphore, // max 2 concurrent requests (spec: politeness)
}

impl Client {
    pub fn new(base: &str) -> Self {
        Self {
            http: std::sync::RwLock::new(Self::build_http()),
            embed_http: with_tls(reqwest::Client::builder())
                .user_agent(UA)
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .expect("embed http client"),
            base: base.trim_end_matches('/').to_string(),
            creds: Mutex::new(None),
            relogin: Mutex::new(()),
            gate: Semaphore::new(2),
        }
    }

    fn build_http() -> reqwest::Client {
        with_tls(reqwest::Client::builder()).cookie_store(true).user_agent(UA).build().expect("http client")
    }

    fn http(&self) -> reqwest::Client {
        self.http.read().expect("http lock").clone()
    }

    /// Forget credentials and drop the session cookies (fresh cookie jar).
    pub async fn logout(&self) {
        self.clear_credentials().await;
        *self.http.write().expect("http lock") = Self::build_http();
    }

    pub async fn set_credentials(&self, user: &str, pass: &str) {
        *self.creds.lock().await = Some((user.into(), pass.into()));
    }

    pub async fn clear_credentials(&self) {
        *self.creds.lock().await = None;
    }

    /// Log in with a fresh cookie jar. Accounts with 2FA (secretkey) are unsupported.
    pub async fn login(&self, user: &str, pass: &str) -> Result<(), Error> {
        let _p = self.gate.acquire().await.expect("semaphore");
        *self.http.write().expect("http lock") = Self::build_http();
        let http = self.http();
        let home = http.get(&self.base).send().await?;
        check_status(home.status())?;
        let action = proxer_parse::login_action(&home.text().await?).ok_or(Error::Parse("login form"))?;
        let res = http
            .post(format!("{}{}", self.base, action))
            .form(&[("username", user), ("password", pass), ("secretkey", ""), ("remember", "1"), ("submit", "Login")])
            .send()
            .await?;
        check_status(res.status())?;
        let res = res.text().await?;
        if proxer_parse::is_logged_in(&res) {
            self.set_credentials(user, pass).await;
            Ok(())
        } else if proxer_parse::login_action(&res).is_some() {
            Err(Error::BadCredentials) // only positive evidence: the login form came back
        } else {
            Err(Error::Parse("login result"))
        }
    }

    async fn fetch(&self, path: &str) -> Result<String, Error> {
        let _p = self.gate.acquire().await.expect("semaphore");
        let res = self.http().get(format!("{}{}", self.base, path)).send().await?;
        check_status(res.status())?; // before the logged-in check: 5xx pages are not a lost session
        Ok(res.text().await?)
    }

    /// GET an embed page anonymously. Only `https://*.proxer.me` is allowed and redirects are not followed.
    pub async fn get_embed(&self, url: &str) -> Result<String, Error> {
        if !is_allowed_embed_url(url) {
            return Err(Error::Parse("embed host not allowed"));
        }
        let _p = self.gate.acquire().await.expect("semaphore");
        // reqwest errors embed the request URL; embed URLs must never leak into logs
        let res = self.embed_http.get(url).send().await.map_err(|e| e.without_url())?;
        check_embed_status(res.status())?;
        Ok(res.text().await.map_err(|e| e.without_url())?)
    }

    /// POST a form to one of Proxer's JSON write endpoints. The URL carries the session token, so errors are
    /// stripped of it. No re-login here: callers GET the token page first (which re-logs in if needed).
    pub async fn post(&self, path: &str, form: &[(&str, &str)]) -> Result<String, Error> {
        let _p = self.gate.acquire().await.expect("semaphore");
        let res = self.http().post(format!("{}{}", self.base, path)).form(form).send().await.map_err(|e| e.without_url())?;
        check_status(res.status())?;
        Ok(res.text().await.map_err(|e| e.without_url())?)
    }

    /// GET a page; if it comes back logged out, log in again once and retry once.
    pub async fn get(&self, path: &str) -> Result<String, Error> {
        let body = self.fetch(path).await?;
        if proxer_parse::is_logged_in(&body) {
            return Ok(body);
        }
        let _one = self.relogin.lock().await;
        // another get may have re-logged-in while we waited
        let body = self.fetch(path).await?;
        if proxer_parse::is_logged_in(&body) {
            return Ok(body);
        }
        let creds = self.creds.lock().await.clone();
        let (u, p) = creds.ok_or(Error::NotLoggedIn)?;
        if let Err(e) = self.login(&u, &p).await {
            return Err(match e {
                Error::BadCredentials => {
                    self.clear_credentials().await; // avoid re-POSTing a bad password (lockout)
                    Error::NotLoggedIn
                }
                other => other,
            });
        }
        let body = self.fetch(path).await?;
        if proxer_parse::is_logged_in(&body) { Ok(body) } else { Err(Error::NotLoggedIn) }
    }
}

fn check_status(s: reqwest::StatusCode) -> Result<(), Error> {
    if s.is_server_error() || s.as_u16() == 429 { Err(Error::Http(s.as_u16())) } else { Ok(()) }
}

/// `https://` and a host that ends in `.proxer.me` (the bare domain and look-alikes like `evilproxer.me` are refused).
pub fn is_allowed_embed_url(url: &str) -> bool {
    let Ok(u) = reqwest::Url::parse(url) else { return false };
    u.scheme() == "https"
        && u.username().is_empty()
        && u.password().is_none()
        && u.port().is_none()
        && u.host_str().is_some_and(|h| h.ends_with(".proxer.me") && !h.starts_with('.'))
}

/// Embed fetches never follow redirects, so 3xx/4xx/5xx are errors, not a page.
fn check_embed_status(s: reqwest::StatusCode) -> Result<(), Error> {
    if s.is_redirection() || s.is_client_error() || s.is_server_error() || s.as_u16() == 429 {
        Err(Error::Http(s.as_u16()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;

    #[test]
    fn embed_status() {
        assert!(check_embed_status(StatusCode::OK).is_ok());
        for c in [301u16, 302, 307, 403, 404, 410, 429, 503] {
            assert!(matches!(check_embed_status(StatusCode::from_u16(c).unwrap()), Err(Error::Http(x)) if x == c), "{c}");
        }
    }
}
