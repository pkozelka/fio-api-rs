//! doc/6: IMPORT (UPLOAD) PLATEBNÍCH PŘÍKAZŮ DO BANK

use std::collections::HashMap;
use std::io::ErrorKind;

use chrono::NaiveDate;

use crate::Result;
use crate::tiny_xml::TinyXml;

// TODO: define strict formats for payments
// TODO: enhance error xml to receive all fields

pub trait ToPaymentXml {
    fn to_payment_xml(&self) -> Result<String>;
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
        self.payment.add_to(&mut doc)?;
        Ok(doc.into_xml()?)
    }
}

/// Support for array of domestic payments
impl ToPaymentXml for &[Payment] {
    fn to_payment_xml(&self) -> Result<String> {
        let mut doc = new_orders_doc()?;
        for payment in self.iter() {
            payment.add_to(&mut doc)?;
        }
        Ok(doc.into_xml()?)
    }
}

/// Name of the special field holding XML type of transaction.
const TRANSACTION_TYPE: &'static str = ".xml_transaction_type";

/// Valid fields and their required order within XML document.
const PAYMENT_FIELDS: &[&str] = &["accountFrom", "currency", "amount", "accountTo",
    "bankCode", "ks", "vs", "ss", "bic", "date", "messageForRecipient", "comment",
    "benefName", "benefStreet", "benefCity", "benefCountry",
    "remittanceInfo1", "remittanceInfo2", "remittanceInfo3", "remittanceInfo4",
    "detailsOfCharges", "paymentReason", "paymentType"];

/// Generalized payment
#[derive(Default, Debug)]
pub struct Payment {
    properties: HashMap<&'static str, String>,
}

impl Payment {
    fn set<S: ToString>(&mut self, key: &'static str, value: S) {
        self.properties.insert(key, value.to_string());
    }

    fn add_to(&self, doc: &mut TinyXml) -> std::io::Result<()> {
        let transaction_xml_type = self.properties.get(TRANSACTION_TYPE)
            .ok_or_else(|| std::io::Error::new(ErrorKind::InvalidData, "Missing transaction type property"))?;

        doc.open(transaction_xml_type)?;
        for &key in PAYMENT_FIELDS {
            match self.properties.get(key) {
                Some(value) if !value.is_empty() => {
                    doc.simple(key, value)?;
                }
                _ => {}
            }
        }
        doc.close()?;
        Ok(())
    }
}

pub trait PaymentBuilder: Sized {
    fn set<S: ToString>(self, key: &'static str, value: S) -> Self;

    fn xml_transaction_type(self, transaction_type: &'static str) -> Self {
        self.set(TRANSACTION_TYPE, transaction_type.to_string())
    }

    /// * `account_from` : (mandatory, 16n) číslo účtu příkazce
    fn account_from<S: ToString>(self, value: S) -> Self {
        self.set("accountFrom", value)
    }

    /// * `currency` : (mandatory, 3!x) měna účtu dle standardu ISO 4217
    fn currency<S: ToString>(self, value: S) -> Self {
        self.set("currency", value)
    }

    /// (mandatory, 18d) částka příkazu
    fn amount(self, value: f64) -> Self {
        self.set("amount", value.to_string())
    }

    /// * `date` : (mandatory, RRRR-MM-DD) datum
    fn date(self, value: NaiveDate) -> Self {
        self.set("date", value.to_string())
    }

    /// helper to set current date
    fn date_today(self) -> Self {
        self.date(chrono::Local::now().date().naive_local())
    }

    /// (optional, 255i) Vaše označení
    fn comment<S: ToString>(self, value: S) -> Self {
        self.set("comment", value)
    }

    /// (optional, 3!n) platební titul – viz 6.3.4 Platební titul
    ///TODO only domestic; t2 has bool:priority; foreign has none
    fn payment_reason(self, value: u16) -> Self {
        self.set("paymentReason", value.to_string())
    }

    /// (optional, 3!n) typ platby; přípustné hodnoty jsou:
    /// * `431001` – standardní
    /// * `431005` – prioritní
    /// * `431022` – příkaz k inkasu
    /// TODO consider using enum here
    fn payment_type(self, value: u32) -> Self {
        self.set("paymentType", value.to_string())
    }
}

pub trait DomesticSymbolsBuilder: PaymentBuilder {
    /// (optional, 4n) konstantní symbol
    fn ks<S: ToString>(self, value: S) -> Self {
        self.set("ks", value)
    }

    /// (optional, 10n) variabilní symbol
    fn vs<S: ToString>(self, value: S) -> Self {
        self.set("vs", value)
    }

    /// (optional, 10n) specifický symbol
    fn ss<S: ToString>(self, value: S) -> Self {
        self.set("ss", value)
    }
}

pub trait DomesticTransaction: PaymentBuilder + DomesticSymbolsBuilder {
    /// * `account_number` : (mandatory, 6n-10n) číslo učtu příjemce/inkasovaného
    /// * `bank_code` : (mandatory, 18d) banka přijemce/inkasovaného
    fn account_to(self, account_number: &str, bank_code: &str) -> Self {
        self
            .set("accountTo", account_number.to_string())
            .set("bankCode", bank_code.to_string())
    }

    /// (optional, 140i) zpráva pro příjemce
    fn message_for_recipient<S: ToString>(self, value: S) -> Self {
        self.set("messageForRecipient", value)
    }
}

/// 6.3.1 XML příkaz platba v rámci ČR
/// Tento pokyn je možné použít i k převodu cízích měn mezi účty v rámci Fio banky.
pub struct DomesticPayment {
    payment: Payment,
}

impl DomesticPayment {
    pub fn new(account_from: &str, currency: &str) -> Self {
        DomesticPayment { payment: Default::default() }
            .xml_transaction_type("DomesticTransaction")
            .account_from(account_from)
            .currency(currency)
            .date_today()
    }
}

impl PaymentBuilder for DomesticPayment {
    fn set<S: ToString>(mut self, key: &'static str, value: S) -> Self {
        self.payment.set(key, value);
        self
    }
}

impl DomesticSymbolsBuilder for DomesticPayment {}

impl DomesticTransaction for DomesticPayment {}

impl From<DomesticPayment> for Payment {
    fn from(dp: DomesticPayment) -> Self {
        dp.payment
    }
}

pub trait AbroadTransaction: PaymentBuilder {
    /// mezinárodní číslo bankovního účtu příjemce/inkasovaného dle standardu ISO 13616
    fn account_to<S: ToString>(self, value: S) -> Self {
        self.set("accountTo", value)
    }

    /// bankovní identifikační kód dle standardu ISO 9362
    fn bic<S: ToString>(self, value: S) -> Self {
        self.set("bic", value)
    }
    fn benef_name<S: ToString>(self, value: S) -> Self {
        self.set("benefName", value)
    }
    fn benef_street<S: ToString>(self, value: S) -> Self {
        self.set("benefName", value)
    }
    fn benef_city<S: ToString>(self, value: S) -> Self {
        self.set("benefName", value)
    }
    fn benef_country<S: ToString>(self, value: S) -> Self {
        self.set("benefName", value)
    }
    fn remittance_info_1<S: ToString>(self, value: S) -> Self {
        self.set("remittanceInfo1", value)
    }
    fn remittance_info_2<S: ToString>(self, value: S) -> Self {
        self.set("remittanceInfo2", value)
    }
    fn remittance_info_3<S: ToString>(self, value: S) -> Self {
        self.set("remittanceInfo3", value)
    }
}

pub trait T2Transaction: AbroadTransaction + DomesticSymbolsBuilder {}

/// 6.3.2 XML příkaz Europlatba
pub struct T2Payment {
    payment: Payment,
}

impl T2Payment {
    pub fn new(account_from: &str, currency: &str) -> Self {
        T2Payment { payment: Default::default() }
            .xml_transaction_type("T2Transaction")
            .account_from(account_from.to_string())
            .currency(currency.to_string())
            .date_today()
    }
}

impl PaymentBuilder for T2Payment {
    fn set<S: ToString>(mut self, key: &'static str, value: S) -> Self {
        self.payment.set(key, value);
        self
    }
}

impl AbroadTransaction for T2Payment {}

impl DomesticSymbolsBuilder for T2Payment {}

impl T2Transaction for T2Payment {}

impl From<T2Payment> for Payment {
    fn from(p: T2Payment) -> Self {
        p.payment
    }
}

/// poplatky
pub enum DetailsOfCharges {
    /// 470501 – vše plátce (OUR)
    OUR,
    /// 470502 – vše přijemce (BEN)
    BEN,
    /// 470503 – každý sám své (SHA)
    SHA,
}

pub trait ForeignTransaction: AbroadTransaction {
    fn remittance_info_4<S: ToString>(self, value: S) -> Self {
        self.set("remittanceInfo4", value)
    }

    fn details_of_charges<S: ToString>(self, details_of_charges: S) -> Self {
        self.set("detailsOfCharges", details_of_charges.to_string())
    }
}

/// 6.3.3 XML příkaz zahraniční platba
pub struct ForeignPayment {
    payment: Payment,
}

impl ForeignPayment {
    pub fn new(account_from: &str, currency: &str) -> Self {
        ForeignPayment { payment: Default::default() }
            .xml_transaction_type("ForeignTransaction")
            .account_from(account_from.to_string())
            .currency(currency.to_string())
            .date_today()
    }
}

impl PaymentBuilder for ForeignPayment {
    fn set<S: ToString>(mut self, key: &'static str, value: S) -> Self {
        self.payment.set(key, value);
        self
    }
}

impl AbroadTransaction for ForeignPayment {}

impl ForeignTransaction for ForeignPayment {}

impl From<ForeignPayment> for Payment {
    fn from(p: ForeignPayment) -> Self {
        p.payment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xyz() {
        let czpayment = Payment::from(DomesticPayment::new("a", "CZK")
            .date_today()
            .amount(3.4)
            .account_to("1234567890", "1234")
            .ks("a")
            .vs("adsf"));
        println!("czpayment: {:?}", czpayment.properties);

        let t2: Payment = T2Payment::new("dsaf", "EUR")
            .date_today()
            .amount(34.1)
            .account_to("fsdafs")
            .bic("dfas")
            .into();
        println!("t2: {:?}", t2.properties);

        let fp = ForeignPayment::new("a", "USD")
            .date_today()
            .amount(43.1)
            .account_to("fadfa")
            .bic("fasdfa");
        println!("fp: {:?}", fp.payment.properties);
    }
}
