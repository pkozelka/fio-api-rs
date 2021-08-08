//! doc/6: IMPORT (UPLOAD) PLATEBNÍCH PŘÍKAZŮ DO BANK

use chrono::NaiveDate;
use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::Response;

use crate::FioClient;

// TODO: support all payment types
// TODO: define strict formats for payments
// TODO: enhance error xml to receive all fields

impl FioClient {
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
            .post(format!("{url_base}/import/", url_base = crate::FIOAPI_URL_BASE))
            .multipart(form)
            .build()?;
        self.client.execute(http_request).await
    }
}

#[derive(Default, Debug)]
struct Payment {
    account_from: String,
    currency: String,
    amount: f64,
    account_to: String,
    bank_code: String,
    ks: String,
    vs: String,
    ss: String,
    date: Option<NaiveDate>,
    message_for_recipient: String,
    comment: String,
}

impl Payment {
    pub fn account_from<S: Into<String>>(mut self, account_from: S) -> Self {
        self.account_from = account_from.into();
        self
    }

    pub fn currency<S: Into<String>>(mut self, currency: S) -> Self {
        self.currency = currency.into();
        self
    }

    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    pub fn account_to<S: Into<String>>(mut self, account_to: S) -> Self {
        self.account_to = account_to.into();
        self
    }

    pub fn bank_code<S: Into<String>>(mut self, bank_code: S) -> Self {
        self.bank_code = bank_code.into();
        self
    }

    pub fn ks<S: Into<String>>(mut self, ks: S) -> Self {
        self.ks = ks.into();
        self
    }

    pub fn vs<S: Into<String>>(mut self, vs: S) -> Self {
        self.vs = vs.into();
        self
    }

    pub fn ss<S: Into<String>>(mut self, ss: S) -> Self {
        self.ss = ss.into();
        self
    }

    pub fn date(mut self, date: NaiveDate) -> Self {
        self.date = Some(date);
        self
    }

    pub fn message_for_recipient<S: Into<String>>(mut self, message_for_recipient: S) -> Self {
        self.message_for_recipient = message_for_recipient.into();
        self
    }

    pub fn comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.comment = comment.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, NaiveDate};

    use crate::FioClient;
    use crate::import::Payment;

    #[test]
    fn test_payment_builder() {
        let p = Payment::default()
            .account_from("1234312")
            .currency("1234234")
            .amount(1234.56)
            .date(Local::now().date().naive_local())
            .account_to("5423");
        let p = p.vs("123");
    }

    #[tokio::test]
    #[ignore]
    async fn test_import() -> anyhow::Result<()> {
        // curl -S --trace-ascii - -X POST -F "type=xml" -F "token=$(cat ../.git/2702016516-hexakoss-rw.txt)" -F "file=@payment.xml" https://www.fio.cz/ib_api/rest/import/

        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::init();

        let token = std::fs::read_to_string(".git/2301479755-OrigisINET-rw.txt")?;
        let payment_xml = std::fs::read_to_string("examples/payment.xml")?;
        let fio = FioClient::new(token.trim());
        println!("R: {}", payment_xml);
        let response = fio.import(&payment_xml).await?;
        println!("R1:");
        let text = response.text().await?;
        println!("Response: {}", text);
        Ok(())
    }
}

// Response: <?xml version="1.0" encoding="UTF-8" standalone="yes"?><responseImport xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://www.fio.cz/schema/responseImport.xsd"><result><errorCode>0</errorCode><idInstruction>1788006308</idInstruction><status>ok</status><sums><sum id="CZK"><sumCredit>0</sumCredit><sumDebet>102.93</sumDebet></sum></sums></result><ordersDetails><detail id="1"><messages><message status="ok" errorCode="0">OK</message></messages></detail></ordersDetails></responseImport>
