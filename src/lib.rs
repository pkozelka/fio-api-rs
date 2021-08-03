//! FIO API library
//! TODO: use our own error enum

use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

pub use error::FioError;
pub use error::Result;
pub use response::FioResponse;
pub use response::FioResponseInfo;

type FioDatum = String;

pub enum FioRequest {}

const FIOAPI_URL_BASE: &'static str = "https://www.fio.cz/ib_api/rest";

/// The low-level client that holds the token.
pub struct FioClient {
    token: String,
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
            client,
        }
    }
}

pub mod csvdata;
pub mod export;
mod import;
mod error;
mod response;
