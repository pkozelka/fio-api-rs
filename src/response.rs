use std::collections::HashMap;
use chrono::{NaiveDate, ParseResult};
use std::num::ParseFloatError;

const DATEFORMAT_DD_MM_YYYY: &str = "%d.%m.%Y";

pub struct FioResponse {
    info_headers: HashMap<String, String>,
}

trait FioResponseInfo {
    fn get_info(&self, key: &str) -> Option<&str>;
}

impl FioResponseInfo for FioResponse {
    fn get_info(&self, key: &str) -> Option<&str> {
        self.info_headers.get(key).map(|s|s.as_str())
    }
}

pub trait FioAccountInfo: FioResponseInfo {
    fn account_id(&self) -> Option<&str> {
        self.get_info("accountId")
    }
    fn bank_id(&self) -> Option<&str> {
        self.get_info("bankId")
    }
    fn iban(&self) -> Option<&str> {
        self.get_info("iban")
    }
    fn bic(&self) -> Option<&str> {
        self.get_info("bic")
    }
}
impl FioAccountInfo for FioResponse {}

pub trait RangeInfo: FioResponseInfo {
    fn opening_balance(&self) -> Result<f64, ParseFloatError> {
        let s = self.get_info("openingBalance")
            .unwrap_or("");
        parse_fio_decimal(s)
    }
    fn closing_balance(&self) -> Result<f64, ParseFloatError> {
        let s = self.get_info("closingBalance")
            .unwrap_or("");
        parse_fio_decimal(s)
    }
    fn date_start(&self) -> Option<ParseResult<NaiveDate>> {
        self.get_info("dateStart").map(parse_fio_date)
    }
    fn date_end(&self) -> Option<ParseResult<NaiveDate>> {
        self.get_info("dateEnd").map(parse_fio_date)
    }

    fn id_from(&self) -> Option<&str> {
        self.get_info("idFrom")
    }
    fn id_to(&self) -> Option<&str> {
        self.get_info("idTo")
    }
}
impl RangeInfo for FioResponse {}

/// Fio uses special decimal format: integer and decimal parts are separated with comma (`,`) instead of dot (`.`).
/// This function resolves the difference.
fn parse_fio_decimal(s: &str) -> Result<f64, ParseFloatError> {
    let s = s.replacen(',', ".", 1); // TODO: get rid of allocation here
    s.parse()
}

fn parse_fio_date(s: &str) -> ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(s, DATEFORMAT_DD_MM_YYYY)
}

impl FioResponse {

}

#[cfg(test)]
mod tests {
    use crate::response::{FioResponse, RangeInfo};
    use std::collections::HashMap;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_balance() {
        let mut info_headers = HashMap::new();
        info_headers.insert("openingBalance".to_string(), "4789,51".to_string());
        let r = FioResponse { info_headers, };
        let balance = r.opening_balance();
        print!("balance = {:?}", balance);
        assert_eq!(Ok(4789.51_f64), balance);
    }

    #[test]
    fn test_parse_date() {
        let mut info_headers = HashMap::new();
        info_headers.insert("dateEnd".to_string(), "31.03.2021".to_string());
        let r = FioResponse { info_headers, };
        let date = r.date_end();
        print!("date = {:?}", date);
        assert_eq!(Some(Ok(NaiveDate::from_ymd(2021, 3, 31))), date);
    }
}
