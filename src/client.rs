use std::cell::Cell;

use reqwest::{Response, StatusCode, Version};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest::multipart::{Form, Part};
use tokio::time::Duration;
use tokio::time::Instant;

use crate::{DomesticPayment, FioExportReq};
use crate::import::{today, ToPaymentXml};

pub(crate) const FIOAPI_URL_BASE: &'static str = "https://www.fio.cz/ib_api/rest";
const REQUEST_RATE: Duration = Duration::from_secs(30);

/// The low-level client that holds the token.
pub struct FioClient {
    token: String,
    last_request: Cell<Instant>,
    client: reqwest::Client,
}

pub struct FioClientWithImport {
    fio: FioClient,
    account_from: String,
    currency: String,
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
            log::trace!("Trying '{}'", fio_req.build_url("__CENSORED__"));
            let http_request = self.client
                .get(fio_req.build_url(&self.token))
                .build()?;
            let response = self.client.execute(http_request).await?;
            match response.status() {
                StatusCode::CONFLICT => self.last_request.set(Instant::now()),
                _ => return response.error_for_status()
            }
        }
    }
}

impl FioClientWithImport {
    /// Construct a client capable of proposing paymetns via the API.
    /// In addition to [`FioClient`], it also needs to know the related account number and currency.
    pub fn new(fio: FioClient, account_from: &str, currency: &str) -> Self {
        Self {
            fio,
            account_from: account_from.to_string(),
            currency: currency.to_string(),
        }
    }

    /// doc/6.1 Import commands - like payments.
    pub async fn import<P: ToPaymentXml>(&self, payment: P) -> reqwest::Result<Response> {
        let payment_xml = payment.to_payment_xml().unwrap();
        log::trace!("payment_xml:\n{}", payment_xml);
        let part = Part::text(payment_xml.to_string())
            .file_name("payments.xml")
            .mime_str("application/xml")?;
        let form = Form::new()
            .text("type", "xml")
            // .text("lng", "en")
            .text("token", self.fio.token.to_string())
            .part("file", part);
        let http_request = self.fio.client
            .post(format!("{url_base}/import/", url_base = FIOAPI_URL_BASE))
            .version(Version::HTTP_11)
            .multipart(form)
            .build()?;
        log::trace!("HTTP Request: {:?}", http_request);
        let response = self.fio.client.execute(http_request).await?;
        response.error_for_status()
        //TODO process response
        // like this:
        // ```
        // <?xml version="1.0" encoding="UTF-8" standalone="yes"?>
        // <responseImport xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://www.fio.cz/schema/responseImport.xsd">
        // <result>
        //   <errorCode>0</errorCode>
        //   <idInstruction>1801400777</idInstruction>
        //   <status>ok</status>
        //   <sums>
        //    <sum id="CZK">
        //     <sumCredit>0</sumCredit>
        //     <sumDebet>444.90</sumDebet>
        //    </sum>
        //   </sums>
        // </result>
        // <ordersDetails>
        //  <detail id="1"><messages><message status="ok" errorCode="0">OK</message></messages></detail>
        //  <detail id="2"><messages><message status="ok" errorCode="0">OK</message></messages></detail>
        // </ordersDetails>
        // </responseImport>
        // ```
    }

    /// Create a domestic transaction with account info pre-filled.
    pub fn new_domestic(&self) -> DomesticPayment {
        DomesticPayment::new(&self.account_from, &self.currency, today())
    }

    /// Create a T2 transaction with account info pre-filled.
    pub fn new_t2(&self) -> () {
        todo!()
    }

    /// Create a foreign transaction with account info pre-filled.
    pub fn new_foreign(&self) -> () {
        todo!()
    }
}

