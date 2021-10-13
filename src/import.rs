//! doc/6: IMPORT (UPLOAD) PLATEBNÍCH PŘÍKAZŮ DO BANK

use std::fmt::Write;

use chrono::NaiveDate;

use crate::FioError;
use crate::Result;

// TODO: support all payment types
// TODO: define strict formats for payments
// TODO: enhance error xml to receive all fields

pub trait ToPaymentXml {
    fn to_payment_xml(&self) -> Result<String>;
}

impl ToPaymentXml for Payment {
    fn to_payment_xml(&self) -> Result<String> {
        let mut out = String::new();
        writeln!(out, r#"<?xml version="1.0" encoding="UTF-8"?>
<Import xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://www.fio.cz/schema/importIB.xsd">
  <Orders>
    <DomesticTransaction>
"#).map_err(|_| FioError::Unknown)?;
        //
        write_elem(&mut out, "accountFrom", &self.account_from)?;
        write_elem(&mut out, "currency", &self.currency)?;
        write_elem(&mut out, "amount", &format!("{}", self.amount))?;
        write_elem(&mut out, "accountTo", &self.account_to)?;
        write_elem(&mut out, "bankCode", &self.bank_code)?;
        write_elem(&mut out, "vs", &self.vs)?;

        //
        writeln!(out, r#"    </DomesticTransaction>
  </Orders>
</Import>
"#).map_err(|_| FioError::Unknown)?;
        Ok(out)
    }
}

fn write_elem(out: &mut String, elem_name: &str, value: &str) -> Result<()> {
    if value.is_empty() {
        return Ok(());
    }
    writeln!(out, "  <{elem_name}>{value}</{elem_name}>",
             elem_name = elem_name,
             value = value, //TODO escape!!!
    ).map_err(|_| FioError::Unknown)
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

#[allow(unused)] //TODO use them
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
    use chrono::Local;

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
        let _ = p.vs("123");
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
