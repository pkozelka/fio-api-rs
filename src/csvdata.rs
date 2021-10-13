use chrono::NaiveDate;
use serde::Deserialize;
use strum_macros::EnumString;
use strum_macros::IntoStaticStr;

#[derive(Debug, Deserialize)]
pub struct FioTransactionsRecord {
    #[serde(rename = "ID pohybu")]
    id_tx: u64,
    #[serde(rename = "Datum", with = "fio_date")]
    date: NaiveDate,
    #[serde(rename = "Objem", with = "fio_decimal")]
    value: f64,
    #[serde(rename = "Měna")]
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

/// 5.1 Podporované formáty dat / Typy pohybů na účtu
#[derive(Debug, Deserialize, IntoStaticStr, EnumString)]
pub enum TxType {
    #[strum(serialize = "Příjem převodem uvnitř banky")]
    FioIncome,
    #[strum(serialize = "Platba převodem uvnitř banky")]
    X02,
    #[strum(serialize = "Vklad pokladnou")]
    X03,
    #[strum(serialize = "Výběr pokladnou")]
    X04,
    #[strum(serialize = "Vklad v hotovosti")]
    X05,
    #[strum(serialize = "Výběr v hotovosti")]
    X06,
    #[strum(serialize = "Platba")]
    X07,
    #[strum(serialize = "Příjem")]
    CardPayment,
    #[strum(serialize = "Bezhotovostní platba")]
    X09,
    #[strum(serialize = "Bezhotovostní příjem")]
    X10,
    #[strum(serialize = "Platba kartou")]
    X11,
    // this variant has duplicate text with X08.CardPayment and can never be instantiated
    // #[strum(serialize = "Bezhotovostní platba")]
    // X12,
    #[strum(serialize = "Úrok z úvěru")]
    X13,
    #[strum(serialize = "Sankční poplatek")]
    X14,
    #[strum(serialize = "Posel – předání")] //TODO attention this is not a dash!
    X15,
    #[strum(serialize = "Posel – příjem")]
    X16,
    #[strum(serialize = "Převod uvnitř konta")]
    X17,
    #[strum(serialize = "Připsaný úrok")]
    X18,
    #[strum(serialize = "Vyplacený úrok")]
    X19,
    #[strum(serialize = "Odvod daně z úroků")]
    X20,
    #[strum(serialize = "Evidovaný úrok")]
    X21,
    #[strum(serialize = "Poplatek")]
    X22,
    #[strum(serialize = "Evidovaný poplatek")]
    X23,
    #[strum(serialize = "Převod mezi bankovními konty (platba)")]
    X24,
    #[strum(serialize = "Převod mezi bankovními konty (příjem)")]
    X25,
    #[strum(serialize = "Neidentifikovaná platba z bankovního konta")]
    X26,
    #[strum(serialize = "Neidentifikovaný příjem na bankovní konto")]
    X27,
    #[strum(serialize = "Vlastní platba z bankovního konta")]
    X28,
    #[strum(serialize = "Vlastní příjem na bankovní konto")]
    X29,
    #[strum(serialize = "Vlastní platba pokladnou")]
    X30,
    #[strum(serialize = "Vlastní příjem pokladnou")]
    X31,
    #[strum(serialize = "Opravný pohyb")]
    X32,
    #[strum(serialize = "Přijatý poplatek")]
    X33,
    #[strum(serialize = "Platba v jiné měně")]
    X34,
    #[strum(serialize = "Poplatek – platební karta")]
    X35,
    #[strum(serialize = "Inkaso")]
    X36,
    #[strum(serialize = "Inkaso ve prospěch účtu")]
    X37,
    #[strum(serialize = "Inkaso z účtu")]
    X38,
    #[strum(serialize = "Příjem inkasa z cizí banky")]
    X39,
    // this variant has duplicate text with X21 and can never be instantiated
    // #[strum(serialize = "Evidovaný úrok")]
    // X40,
    #[strum(serialize = "Okamžitá příchozí platba")]
    X41,
    #[strum(serialize = "Okamžitá odchozí platba")]
    X42,
    #[strum(serialize = "Poplatek - pojištění hypotéky")]
    X43,
    //
    Other(String),
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

    mod tests {
        #[test]
        fn test_parse_fio_date() -> crate::Result<()> {
            let date = super::parse_fio_date("30.06.2021")?;
            println!("date = {:?}", date);
            assert_eq!(chrono::NaiveDate::from_ymd(2021, 6, 30), date);
            Ok(())
        }
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
    use std::str::FromStr;

    use serde::{Deserialize, Deserializer};

    use crate::csvdata::TxType;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<TxType, D::Error>
        where D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        TxType::from_str(&s).or(Ok(TxType::Other(s)))
    }
}

