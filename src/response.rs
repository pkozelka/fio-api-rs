use std::collections::HashMap;
use chrono::{NaiveDate, ParseResult};
use std::num::ParseFloatError;
use reqwest::Response;
use crate::FioError;
use crate::error::parse_xml_error;
use crate::error::FioError::OtherError;
use csv::StringRecord;

const DATEFORMAT_DD_MM_YYYY: &str = "%d.%m.%Y";

pub struct FioResponse {
    info_headers: HashMap<String, String>,
}

pub trait FioResponseInfo {
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

pub trait FioRangeInfo: FioResponseInfo {
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
impl FioRangeInfo for FioResponse {}

/// Fio uses special decimal format: integer and decimal parts are separated with comma (`,`) instead of dot (`.`).
/// This function resolves the difference.
pub(crate) fn parse_fio_decimal(s: &str) -> Result<f64, ParseFloatError> {
    let s = s.replacen(',', ".", 1); // TODO: get rid of allocation here
    s.parse()
}

pub(crate) fn parse_fio_date(s: &str) -> ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(s, DATEFORMAT_DD_MM_YYYY)
}

impl FioResponse {
    pub async fn try_from(response: Response) -> crate::Result<Self> {
        // analyze HTTP headers
        let text = match response.status().as_u16() {
            200..=299 => Ok(response.text().await?),
            404 => Err(FioError::BadRequest),
            409 => Err(FioError::InvalidTiming),
            413 => Err(FioError::TooManyRows),
            500 => Err(parse_xml_error(response).await),
            _ => Err(OtherError { code: "other".to_string(), message: response.status().canonical_reason().unwrap_or("?").to_string()})
        }?;
        let mut this = FioResponse { info_headers: HashMap::new() };
        // prepare csv reader
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .has_headers(false)
            .flexible(true)
            .from_reader(text.as_bytes());
        this.csv_fetch_info_lines(&mut csv_reader).await?;
        // create object capable of iterating over the actual data
        //TODO
        Ok(this)
    }

    async fn csv_fetch_info_lines<R: std::io::Read> (&mut self, csv_reader: &mut csv::Reader<R>) -> crate::Result<()> {
        // parse and store initial data info
        let mut record = StringRecord::new();
        while csv_reader.read_record(&mut record)? {
            match record.len() {
                2 => {
                    // the heading part
                    let key = record.get(0).unwrap();
                    let value = record.get(1).unwrap();
                    self.info_headers.insert(key.to_string(), value.to_string());
                },
                0 => {
                    // skip all separator lines
                    log::trace!("(-- separator line --)");
                    while record.len() == 0 {
                        if !csv_reader.read_record(&mut record)? {
                            break;
                        }
                    }
                    break;
                },
                _ => {
                    log::debug!("Column count: {}", record.len());
                    break;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::response::{FioResponse, FioRangeInfo};
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
