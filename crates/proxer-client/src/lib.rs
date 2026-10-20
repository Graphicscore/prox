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
    #[error("network: {0}")]
    Network(#[from] reqwest::Error),
}

pub struct Client {
    http: reqwest::Client,
    base: String,
    creds: Mutex<Option<(String, String)>>,
    relogin: Mutex<()>, // single-flight re-login; taken before any semaphore permit
    gate: Semaphore, // max 2 concurrent requests (spec: politeness)
}

impl Client {
    pub fn new(base: &str) -> Self {
        Self {
            http: reqwest::Client::builder().cookie_store(true).user_agent(UA).build().expect("http client"),
            base: base.trim_end_matches('/').to_string(),
            creds: Mutex::new(None),
            relogin: Mutex::new(()),
            gate: Semaphore::new(2),
        }
    }

    pub async fn set_credentials(&self, user: &str, pass: &str) {
        *self.creds.lock().await = Some((user.into(), pass.into()));
    }

    pub async fn clear_credentials(&self) {
        *self.creds.lock().await = None;
    }

    pub async fn login(&self, user: &str, pass: &str) -> Result<(), Error> {
        let _p = self.gate.acquire().await.expect("semaphore");
        let home = self.http.get(&self.base).send().await?.text().await?;
        let action = proxer_parse::login_action(&home).ok_or(Error::Parse("login form"))?;
        let res = self
            .http
            .post(format!("{}{}", self.base, action))
            .form(&[("username", user), ("password", pass)])
            .send()
            .await?
            .text()
            .await?;
        if proxer_parse::is_logged_in(&res) {
            self.set_credentials(user, pass).await;
            Ok(())
        } else {
            Err(Error::BadCredentials)
        }
    }

    async fn fetch(&self, path: &str) -> Result<String, Error> {
        let _p = self.gate.acquire().await.expect("semaphore");
        Ok(self.http.get(format!("{}{}", self.base, path)).send().await?.text().await?)
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
