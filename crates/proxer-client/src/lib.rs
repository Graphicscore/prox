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

pub struct Client {
    http: std::sync::RwLock<reqwest::Client>,
    base: String,
    creds: Mutex<Option<(String, String)>>,
    relogin: Mutex<()>, // single-flight re-login; taken before any semaphore permit
    gate: Semaphore, // max 2 concurrent requests (spec: politeness)
}

impl Client {
    pub fn new(base: &str) -> Self {
        Self {
            http: std::sync::RwLock::new(Self::build_http()),
            base: base.trim_end_matches('/').to_string(),
            creds: Mutex::new(None),
            relogin: Mutex::new(()),
            gate: Semaphore::new(2),
        }
    }

    fn build_http() -> reqwest::Client {
        reqwest::Client::builder().cookie_store(true).user_agent(UA).build().expect("http client")
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
