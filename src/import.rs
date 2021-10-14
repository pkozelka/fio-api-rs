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
        write!(out, r#"<?xml version="1.0" encoding="UTF-8"?>
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
        write_elem(&mut out, "ks", &self.ks)?;
        write_elem(&mut out, "vs", &self.vs)?;
        write_elem(&mut out, "ss", &self.ss)?;
        write_elem(&mut out, "date", &self.date.unwrap().to_string())?;
        write_elem(&mut out, "messageForRecipient", &self.message_for_recipient)?;
        write_elem(&mut out, "comment", &self.comment)?;

        //
        writeln!(out, r#"</DomesticTransaction>
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

#[allow(unused)] //TODO use them
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
