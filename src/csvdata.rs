use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FioTransactionsRecord {
    #[serde(rename="ID pohybu")]
    id_tx: u64,
    #[serde(rename="Datum", with="fio_date")]
    date: NaiveDate,
    #[serde(rename="Objem", with="fio_decimal")]
    value: f64,
    #[serde(rename="Měna")]
    currency: String,
    #[serde(rename="Protiúčet")]
    b_account: String,
    #[serde(rename="Název protiúčtu")]
    b_account_name: String,
    #[serde(rename="Kód banky")]
    b_bankid: String,
    #[serde(rename="Název banky")]
    b_bank_name: Option<String>,
    #[serde(rename="KS")]
    ks: String,
    #[serde(rename="VS")]
    vs: String,
    #[serde(rename="SS")]
    ss: String,
    #[serde(rename="Uživatelská identifikace")]
    custom_id: String,
    #[serde(rename="Zpráva pro příjemce")]
    message: String,
    #[serde(rename="Typ", with="fio_txtype")]
    tx_type: TxType,
    #[serde(rename="Provedl")]
    who: String,
    #[serde(rename="Upřesnění")]
    note: String,
    #[serde(rename="Komentář")]
    comment: String,
    #[serde(rename="BIC")]
    bic: String,
    #[serde(rename="ID pokynu")]
    id_command: u64,
}

#[derive(Debug, Deserialize)]
pub enum TxType {
    // Platba kartou
    CardPayment,
    // Příjem převodem uvnitř banky
    FioIncome,
    //
    Other(String)
}

pub(crate) mod fio_date {
    use chrono::{NaiveDate, ParseResult};
    use serde::{Deserialize, Deserializer};

    const DATEFORMAT_DD_MM_YYYY: &str = "%d.%m.%Y";

    pub fn parse_fio_date(s: &str) -> ParseResult<NaiveDate> {
        NaiveDate::parse_from_str(s, DATEFORMAT_DD_MM_YYYY)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
        where D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_fio_date(&s)
            .map_err(serde::de::Error::custom)
    }
}

pub(crate) mod fio_decimal {
    use std::num::ParseFloatError;

    use serde::{Deserialize, Deserializer};

    /// Fio uses special decimal format: integer and decimal parts are separated with comma (`,`) instead of dot (`.`).
    /// This function resolves the difference.
    pub fn parse_fio_decimal(s: &str) -> Result<f64, ParseFloatError> {
        let s = s.replacen(',', ".", 1); // TODO: get rid of allocation here
        s.parse()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
        where D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        parse_fio_decimal(&s)
            .map_err(serde::de::Error::custom)
    }
}

mod fio_txtype {
    use serde::{Deserialize, Deserializer};

    use crate::csvdata::TxType;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<TxType, D::Error>
        where D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Platba kartou" => Ok(TxType::CardPayment),
            "Příjem převodem uvnitř banky" => Ok(TxType::FioIncome),
            //TODO use some clever macro to support all values
            _ => Ok(TxType::Other(s))
        }
    }
}

