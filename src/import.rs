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

pub(crate) fn today() -> NaiveDate {
    chrono::Local::now().date().naive_local()
}


/// XML příkaz platba v rámci ČR
/// Tento pokyn je možné použít i k převodu cízích měn mezi účty v rámci Fio banky.
#[derive(Debug)]
pub struct DomesticPayment {
    account_from: String,
    currency: String,
    amount: f64,
    account_to: String,
    bank_code: String,
    ks: String,
    vs: String,
    ss: String,
    date: NaiveDate,
    message_for_recipient: String,
    comment: String,
    payment_reason: Option<u16>,
    payment_type: Option<u32>,
}

impl Default for DomesticPayment {
    fn default() -> Self {
        Self {
            account_from: Default::default(),
            currency: Default::default(),
            amount: f64::NAN,
            account_to: Default::default(),
            bank_code: Default::default(),
            ks: Default::default(),
            vs: Default::default(),
            ss: Default::default(),
            date: today(),
            message_for_recipient: Default::default(),
            comment: Default::default(),
            payment_reason: None,
            payment_type: None,
        }
    }
}

impl DomesticPayment {
    /// * `account_from` : (mandatory, 16n) číslo účtu příkazce
    /// * `currency` : (mandatory, 3!x) měna účtu dle standardu ISO 4217
    /// * `date` : (mandatory, RRRR-MM-DD) datum
    pub fn new(account_from: &str, currency: &str, date: NaiveDate) -> Self {
        Self {
            account_from: account_from.to_string(),
            currency: currency.to_string(),
            date,
            ..Default::default()
        }
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

    fn add_to(&self, doc: &mut TinyXml) -> Result<()> {
        doc.open("DomesticTransaction")?;

        doc.simple("accountFrom", &self.account_from)?;
        doc.simple("currency", &self.currency)?;
        doc.simple("amount", &format!("{}", self.amount))?;
        doc.simple("accountTo", &self.account_to)?;
        doc.simple("bankCode", &self.bank_code)?;
        doc.simple("ks", &self.ks)?;
        doc.simple("vs", &self.vs)?;
        doc.simple("ss", &self.ss)?;
        doc.simple("date", &self.date.to_string())?;
        doc.simple("messageForRecipient", &self.message_for_recipient)?;
        doc.simple("comment", &self.comment)?;

        doc.close()?;
        Ok(())
    }
}

/// Instantiates a new XML document for import of payment orders.
fn new_orders_doc() -> Result<TinyXml> {
    let mut doc = TinyXml::new()?;
    doc.open_attrs("Import", &[
        ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
        ("xsi:noNamespaceSchemaLocation", "http://www.fio.cz/schema/importIB.xsd"),
    ])?;
    doc.open("Orders")?;
    Ok(doc)
}

/// Support for single domestic payment
impl ToPaymentXml for DomesticPayment {
    fn to_payment_xml(&self) -> Result<String> {
        let mut doc = new_orders_doc()?;
        self.add_to(&mut doc)?;
        Ok(doc.into_xml()?)
    }
}

/// Support for array of domestic payments
impl ToPaymentXml for &[DomesticPayment] {
    fn to_payment_xml(&self) -> Result<String> {
        let mut doc = new_orders_doc()?;
        for payment in self.iter() {
            payment.add_to(&mut doc)?;
        }
        Ok(doc.into_xml()?)
    }
}

