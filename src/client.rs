use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use reqwest::{
    header::{self},
    Response,
};
use serde::de::DeserializeOwned;

use crate::{
    blob,
    core::{
        request::{self, Request},
        response,
        session::{Session, URLPart},
    },
    event_source, Error,
};

const DEFAULT_TIMEOUT_MS: u64 = 10 * 1000;
static USER_AGENT: &str = concat!("stalwart-jmap/", env!("CARGO_PKG_VERSION"));

pub enum Credentials {
    Basic(String),
    Bearer(String),
}

pub struct Client {
    session: Session,
    session_url: String,
    session_outdated: AtomicBool,
    #[cfg(feature = "websockets")]
    pub(crate) authorization: String,
    upload_url: Vec<URLPart<blob::URLParameter>>,
    download_url: Vec<URLPart<blob::URLParameter>>,
    event_source_url: Vec<URLPart<event_source::URLParameter>>,
    timeout: u64,
    headers: header::HeaderMap,
    default_account_id: String,
    #[cfg(feature = "websockets")]
    pub(crate) ws: tokio::sync::Mutex<Option<crate::client_ws::WsStream>>,
}

impl Client {
    pub async fn connect(url: &str, credentials: impl Into<Credentials>) -> crate::Result<Self> {
        let authorization = match credentials.into() {
            Credentials::Basic(s) => format!("Basic {}", s),
            Credentials::Bearer(s) => format!("Bearer {}", s),
        };
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(USER_AGENT),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&authorization).unwrap(),
        );

        let session: Session = serde_json::from_slice(
            &Client::handle_error(
                reqwest::Client::builder()
                    .timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
                    .default_headers(headers.clone())
                    .build()?
                    .get(url)
                    .send()
                    .await?,
            )
            .await?
            .bytes()
            .await?,
        )?;

        let default_account_id = session
            .primary_accounts()
            .next()
            .map(|a| a.1.to_string())
            .unwrap_or_default();

        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        Ok(Client {
            download_url: URLPart::parse(session.download_url())?,
            upload_url: URLPart::parse(session.upload_url())?,
            event_source_url: URLPart::parse(session.event_source_url())?,
            session,
            session_url: url.to_string(),
            session_outdated: false.into(),
            #[cfg(feature = "websockets")]
            authorization,
            timeout: DEFAULT_TIMEOUT_MS,
            headers,
            default_account_id,
            #[cfg(feature = "websockets")]
            ws: None.into(),
        })
    }

    pub fn set_timeout(&mut self, timeout: u64) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn timeout(&self) -> u64 {
        self.timeout
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn session_url(&self) -> &str {
        &self.session_url
    }

    pub fn headers(&self) -> &header::HeaderMap {
        &self.headers
    }

    pub async fn send<R>(
        &self,
        request: &request::Request<'_>,
    ) -> crate::Result<response::Response<R>>
    where
        R: DeserializeOwned,
    {
        let response: response::Response<R> = serde_json::from_slice(
            &Client::handle_error(
                reqwest::Client::builder()
                    .timeout(Duration::from_millis(self.timeout))
                    .default_headers(self.headers.clone())
                    .build()?
                    .post(self.session.api_url())
                    .body(serde_json::to_string(&request)?)
                    .send()
                    .await?,
            )
            .await?
            .bytes()
            .await?,
        )?;

        if response.session_state() != self.session.state() {
            self.session_outdated.store(true, Ordering::Relaxed);
        }

        Ok(response)
    }

    pub async fn refresh_session(&mut self) -> crate::Result<()> {
        let session: Session = serde_json::from_slice(
            &Client::handle_error(
                reqwest::Client::builder()
                    .timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
                    .default_headers(self.headers.clone())
                    .build()?
                    .get(&self.session_url)
                    .send()
                    .await?,
            )
            .await?
            .bytes()
            .await?,
        )?;
        self.download_url = URLPart::parse(session.download_url())?;
        self.upload_url = URLPart::parse(session.upload_url())?;
        self.event_source_url = URLPart::parse(session.event_source_url())?;
        self.session = session;
        self.session_outdated.store(false, Ordering::Relaxed);
        Ok(())
    }

    pub fn is_session_updated(&self) -> bool {
        !self.session_outdated.load(Ordering::Relaxed)
    }

    pub fn set_default_account_id(&mut self, defaul_account_id: impl Into<String>) -> &mut Self {
        self.default_account_id = defaul_account_id.into();
        self
    }

    pub fn default_account_id(&self) -> &str {
        &self.default_account_id
    }

    pub fn build(&self) -> Request<'_> {
        Request::new(self)
    }

    pub fn download_url(&self) -> &[URLPart<blob::URLParameter>] {
        &self.download_url
    }

    pub fn upload_url(&self) -> &[URLPart<blob::URLParameter>] {
        &self.upload_url
    }

    pub fn event_source_url(&self) -> &[URLPart<event_source::URLParameter>] {
        &self.event_source_url
    }

    pub async fn handle_error(response: Response) -> crate::Result<Response> {
        if response.status().is_success() {
            Ok(response)
        } else if let Some(b"application/problem+json") = response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|h| h.as_bytes())
        {
            Err(Error::Problem(serde_json::from_slice(
                &response.bytes().await?,
            )?))
        } else {
            Err(Error::Server(format!("{}", response.status())))
        }
    }
}

impl Credentials {
    pub fn basic(username: &str, password: &str) -> Self {
        Credentials::Basic(base64::encode(format!("{}:{}", username, password)))
    }

    pub fn bearer(token: impl Into<String>) -> Self {
        Credentials::Bearer(token.into())
    }
}

impl From<&str> for Credentials {
    fn from(s: &str) -> Self {
        Credentials::bearer(s.to_string())
    }
}

impl From<String> for Credentials {
    fn from(s: String) -> Self {
        Credentials::bearer(s)
    }
}

impl From<(&str, &str)> for Credentials {
    fn from((username, password): (&str, &str)) -> Self {
        Credentials::basic(username, password)
    }
}

impl From<(String, String)> for Credentials {
    fn from((username, password): (String, String)) -> Self {
        Credentials::basic(&username, &password)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::response::{Response, TaggedMethodResponse};

    #[test]
    fn test_deserialize() {
        let _r: Response<TaggedMethodResponse> = serde_json::from_slice(
            br#"{"sessionState": "123", "methodResponses": [[ "Email/query", {
                "accountId": "A1",
                "queryState": "abcdefg",
                "canCalculateChanges": true,
                "position": 0,
                "total": 101,
                "ids": [ "msg1023", "msg223", "msg110", "msg93", "msg91",
                    "msg38", "msg36", "msg33", "msg11", "msg1" ]
            }, "t0" ],
            [ "Email/get", {
                "accountId": "A1",
                "state": "123456",
                "list": [{
                    "id": "msg1023",
                    "threadId": "trd194"
                }, {
                    "id": "msg223",
                    "threadId": "trd114"
                }
                ],
                "notFound": []
            }, "t1" ],
            [ "Thread/get", {
                "accountId": "A1",
                "state": "123456",
                "list": [{
                    "id": "trd194",
                    "emailIds": [ "msg1020", "msg1021", "msg1023" ]
                }, {
                    "id": "trd114",
                    "emailIds": [ "msg201", "msg223" ]
                }
                ],
                "notFound": []
            }, "t2" ]]}"#,
        )
        .unwrap();

        //println!("{:?}", r);
    }
}
