//! doc/6: IMPORT (UPLOAD) PLATEBNÍCH PŘÍKAZŮ DO BANK

use chrono::NaiveDate;

use crate::Result;
use crate::tiny_xml::TinyXml;

// TODO: support all payment types
// TODO: define strict formats for payments
// TODO: enhance error xml to receive all fields

pub trait ToPaymentXml {
    fn to_payment_xml(&self) -> Result<String>;
}

impl ToPaymentXml for Payment {
    fn to_payment_xml(&self) -> Result<String> {
        let mut doc = TinyXml::new()?;
        doc.open_attrs("Import", &[
            ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
            ("xsi:noNamespaceSchemaLocation", "http://www.fio.cz/schema/importIB.xsd"),
        ])?;
        doc.open("Orders")?;
        doc.open("DomesticTransaction")?;

        doc.simple("accountFrom", &self.account_from)?;
        doc.simple("currency", &self.currency)?;
        doc.simple("amount", &format!("{}", self.amount))?;
        doc.simple("accountTo", &self.account_to)?;
        doc.simple("bankCode", &self.bank_code)?;
        doc.simple("ks", &self.ks)?;
        doc.simple("vs", &self.vs)?;
        doc.simple("ss", &self.ss)?;
        doc.simple("date", &self.date.unwrap().to_string())?;
        doc.simple("messageForRecipient", &self.message_for_recipient)?;
        doc.simple("comment", &self.comment)?;

        Ok(doc.into_xml()?)
    }
}

#[derive(Default, Debug)]
pub struct Payment {
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
    payment_reason: Option<u16>,
    payment_type: Option<u32>,
}

impl Payment {
    /// (mandatory, 16n) číslo účtu příkazce
    pub fn account_from<S: Into<String>>(mut self, account_from: S) -> Self {
        self.account_from = account_from.into();
        self
    }

    /// (mandatory, 3!x) měna účtu dle standardu ISO 4217
    pub fn currency<S: Into<String>>(mut self, currency: S) -> Self {
        self.currency = currency.into();
        self
    }

    /// (mandatory, 18d) částka příkazu
    pub fn amount(mut self, amount: f64) -> Self {
        self.amount = amount;
        self
    }

    /// (mandatory, 6n-10n) číslo učtu příjemce/inkasovaného
    pub fn account_to<S: Into<String>>(mut self, account_to: S) -> Self {
        self.account_to = account_to.into();
        self
    }

    /// (mandatory, 18d) banka přijemce/inkasovaného
    pub fn bank_code<S: Into<String>>(mut self, bank_code: S) -> Self {
        self.bank_code = bank_code.into();
        self
    }

    /// (optional, 4n) konstantní symbol
    pub fn ks<S: Into<String>>(mut self, ks: S) -> Self {
        self.ks = ks.into();
        self
    }

    /// (optional, 10n) variabilní symbol
    pub fn vs<S: Into<String>>(mut self, vs: S) -> Self {
        self.vs = vs.into();
        self
    }

    /// (optional, 10n) specifický symbol
    pub fn ss<S: Into<String>>(mut self, ss: S) -> Self {
        self.ss = ss.into();
        self
    }

    /// (mandatory, RRRR-MM-DD) datum
    pub fn date(mut self, date: NaiveDate) -> Self {
        self.date = Some(date);
        self
    }

    /// (optional, 140i) zpráva pro příjemce
    pub fn message_for_recipient<S: Into<String>>(mut self, message_for_recipient: S) -> Self {
        self.message_for_recipient = message_for_recipient.into();
        self
    }

    /// (optional, 255i) Vaše označení
    pub fn comment<S: Into<String>>(mut self, comment: S) -> Self {
        self.comment = comment.into();
        self
    }

    /// (optional, 3!n) platební titul – viz 6.3.4 Platební titul
    pub fn payment_reason(mut self, payment_reason: u16) -> Self {
        self.payment_reason = Some(payment_reason);
        self
    }

    /// (optional, 3!n) typ platby; přípustné hodnoty jsou:
    /// * `431001` – standardní
    /// * `431005` – prioritní
    /// * `431022` – příkaz k inkasu
    /// TODO consider using enum here
    pub fn payment_type(mut self, payment_type: u32) -> Self {
        self.payment_type = Some(payment_type);
        self
    }
}
