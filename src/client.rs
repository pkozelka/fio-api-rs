use std::cell::Cell;

use reqwest::{Response, StatusCode};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::multipart::{Form, Part};
use tokio::time::Duration;
use tokio::time::Instant;

use crate::FioExportReq;

pub(crate) const FIOAPI_URL_BASE: &'static str = "https://www.fio.cz/ib_api/rest";
const REQUEST_RATE: Duration = Duration::from_secs(30);

/// The low-level client that holds the token.
pub struct FioClient {
    token: String,
    last_request: Cell<Instant>,
    client: reqwest::Client,
}

impl FioClient {
    pub fn new(token: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("fio-api-rs"));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build().unwrap();
        Self {
            token: token.to_string(),
            last_request: Cell::new(Instant::now() - REQUEST_RATE),
            client,
        }
    }

    /// Read-only commands.
    /// See methods in [FioExportReq] for commands that can be used here.
    pub async fn export(&self, fio_req: FioExportReq) -> reqwest::Result<Response> {
        loop {
            let next_time = self.last_request.get() + REQUEST_RATE;
            let now = Instant::now();
            if now < next_time {
                log::trace!("Delaying next call to FIO API; duration {}", next_time.duration_since(now).as_millis());
                tokio::time::sleep_until(next_time).await;
                self.last_request.set(next_time);
            }
            let http_request = self.client
                .get(fio_req.build_url(&self.token))
                .build()?;
            let response = self.client.execute(http_request).await?;
            match response.status() {
                StatusCode::OK => {
                    return Ok(response);
                }
                StatusCode::CONFLICT => {
                    let censored_cmd = fio_req.build_url("*CENSORED*");
                    log::warn!("Retrying command '{}'", censored_cmd);
                    self.last_request.set(Instant::now());
                }
                _ => {
                    return response.error_for_status();
                }
            }
        }
    }

    /// Import commands - like payments.
    /// TODO: make symmetrical to [export].
    pub async fn import(&self, payment_xml: &str) -> reqwest::Result<Response> {
        // doc/6.1
        let part = Part::text(payment_xml.to_string())
            .file_name("payments.xml")
            .mime_str("application/xml")?;
        let form = Form::new()
            .text("type", "xml")
            .text("token", self.token.to_string())
            .part("file", part)
            .text("lng", "en");
        let http_request = self.client
            .post(format!("{url_base}/import/", url_base = FIOAPI_URL_BASE))
            .multipart(form)
            .build()?;
        self.client.execute(http_request).await
    }
}

