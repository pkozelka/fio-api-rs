use std::collections::HashMap;
use std::io::{BufRead, Cursor, ErrorKind};
use std::num::ParseFloatError;

use chrono::{NaiveDate, ParseResult};
use csv::{DeserializeRecordsIntoIter, Reader};
use reqwest::Response;

use crate::csvdata::FioTransactionsRecord;
use crate::error::FioError::OtherError;
use crate::error::parse_xml_error;
use crate::FioError;

const DATEFORMAT_DD_MM_YYYY: &str = "%d.%m.%Y";

pub struct FioResponse {
    cursor: Cursor<Vec<u8>>,
}

impl From<Cursor<Vec<u8>>> for FioResponse {
    fn from(cursor: Cursor<Vec<u8>>) -> Self {
        Self { cursor }
    }
}

impl FioResponse {
    pub fn new(cursor: Cursor<Vec<u8>>) -> std::io::Result<Self> {
        let mut this = Self::from(cursor);
        FioResponseInfo::skip(&mut this.cursor)?;
        Ok(this)
    }

    pub async fn try_from(response: Response) -> crate::Result<Self> {
        // analyze HTTP headers
        match response.status().as_u16() {
            200..=299 => {
                let bytes = response.bytes().await?.to_vec();
                let cursor = Cursor::new(bytes);
                Ok(Self::from(cursor))
            },
            404 => Err(FioError::BadRequest),
            409 => Err(FioError::InvalidTiming),
            413 => Err(FioError::TooManyRows),
            500 => Err(parse_xml_error(response).await),
            _ => Err(OtherError { code: "other".to_string(), message: response.status().canonical_reason().unwrap_or("?").to_string() })
        }
    }

    pub fn info(&mut self) -> std::io::Result<FioResponseInfo> {
        if self.cursor.position() > 0 {
            return Err(std::io::Error::new(ErrorKind::Unsupported, "Info part was already read from the stream"));
        }
        FioResponseInfo::read(&mut self.cursor)
    }

    pub fn csv_reader(mut self) -> std::io::Result<Reader<Cursor<Vec<u8>>>> {
        FioResponseInfo::skip(&mut self.cursor)?;
        Ok(csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(self.cursor))
    }

    pub fn data(self) -> std::io::Result<DeserializeRecordsIntoIter<Cursor<Vec<u8>>,FioTransactionsRecord>> {
        let csv_reader = self.csv_reader()?;
        Ok(csv_reader.into_deserialize())
    }

    pub fn into_inner(self) -> Cursor<Vec<u8>> {
        self.cursor
    }

    pub fn get_ref(&self) -> &Cursor<Vec<u8>> {
        &self.cursor
    }
}

/// Representation of the initial "info" part of FIO csv file.
pub struct FioResponseInfo {
    info_headers: HashMap<String, String>,
}

impl From<HashMap<String, String>> for FioResponseInfo {
    fn from(info_headers: HashMap<String, String>) -> Self {
        Self { info_headers }
    }
}

impl FioResponseInfo {
    /// Read from cursor
    pub fn read(cursor: &mut Cursor<Vec<u8>>) -> std::io::Result<Self> {
        let mut line = String::new();
        let mut info_headers = HashMap::new();
        while cursor.read_line(&mut line)? > 0 {
            if line.starts_with('\u{feff}') {
                // remove BOM
                line.remove(0);
            }
            match line.find(';') {
                None => break,
                Some(n) => {
                    let key = &line[0..n];
                    let value = line[n + 1..].trim_end();
                    info_headers.insert(key.to_string(), value.to_string());
                }
            }
            line.clear();
        }
        Ok(Self::from(info_headers))
    }

    pub fn skip(cursor: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
        // if position is not 0, we suppose that info part was already read
        if cursor.position() == 0 {
            let mut line = String::new();
            while cursor.read_line(&mut line)? > 0 {
                if let None = line.find(';') {
                    break;
                }
                line.clear();
            }
        }
        Ok(())
    }

    fn get_info(&self, key: &str) -> Option<&str> {
        self.info_headers.get(key).map(|s| s.as_str())
    }

    /// Consumes instance, returning internal representation of the response info.
    pub fn into_inner(self) -> HashMap<String, String> {
        self.info_headers
    }

    /// Returns reference to internal representation of the response info.
    pub fn get_ref(&self) -> &HashMap<String, String> {
        &self.info_headers
    }

    pub fn account_id(&self) -> Option<&str> {
        self.get_info("accountId")
    }

    pub fn bank_id(&self) -> Option<&str> {
        self.get_info("bankId")
    }

    pub fn iban(&self) -> Option<&str> {
        self.get_info("iban")
    }

    pub fn bic(&self) -> Option<&str> {
        self.get_info("bic")
    }

    pub fn opening_balance(&self) -> Result<f64, ParseFloatError> {
        let s = self.get_info("openingBalance")
            .unwrap_or("");
        parse_fio_decimal(s)
    }

    pub fn closing_balance(&self) -> Result<f64, ParseFloatError> {
        let s = self.get_info("closingBalance")
            .unwrap_or("");
        parse_fio_decimal(s)
    }

    pub fn date_start(&self) -> Option<ParseResult<NaiveDate>> {
        self.get_info("dateStart").map(parse_fio_date)
    }

    pub fn date_end(&self) -> Option<ParseResult<NaiveDate>> {
        self.get_info("dateEnd").map(parse_fio_date)
    }

    pub fn id_from(&self) -> Option<&str> {
        self.get_info("idFrom")
    }
    pub fn id_to(&self) -> Option<&str> {
        self.get_info("idTo")
    }
}

/// Fio uses special decimal format: integer and decimal parts are separated with comma (`,`) instead of dot (`.`).
/// This function resolves the difference.
pub(crate) fn parse_fio_decimal(s: &str) -> Result<f64, ParseFloatError> {
    let s = s.replacen(',', ".", 1); // TODO: get rid of allocation here
    s.parse()
}

pub(crate) fn parse_fio_date(s: &str) -> ParseResult<NaiveDate> {
    NaiveDate::parse_from_str(s, DATEFORMAT_DD_MM_YYYY)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use chrono::NaiveDate;

    use crate::error::Result;
    use crate::FioResponseInfo;
    use crate::response::parse_fio_date;

    const SAMPLE1: &str = r#"accountId;2345678901
bankId;2010
currency;CZK
iban;CZ6220100000002345678901
bic;FIOBCZPPXXX
openingBalance;4789,51
closingBalance;19753,26
dateStart;01.06.2021
dateEnd;30.06.2021
yearList;2021
idList;6
idFrom;23771345451
idTo;23794028126

ID pohybu;Datum;Objem;Měna;Protiúčet;Název protiúčtu;Kód banky;Název banky;KS;VS;SS;Uživatelská identifikace;Zpráva pro příjemce;Typ;Provedl;Upřesnění;Komentář;BIC;ID pokynu
"#;

    impl FioResponseInfo {
        fn sample1() -> Result<Self> {
            let mut cursor = Cursor::new(SAMPLE1.as_bytes().to_vec());
            Ok(FioResponseInfo::read(&mut cursor)?)
        }
    }

    #[test]
    fn test_parse_balance() -> Result<()> {
        let info = FioResponseInfo::sample1()?;
        let balance = info.opening_balance();
        println!("balance = {:?}", balance);
        assert_eq!(Ok(4789.51_f64), balance);
        Ok(())
    }

    #[test]
    fn test_parse_date() -> Result<()> {
        let info = FioResponseInfo::sample1()?;
        let date = info.date_end();
        println!("date = {:?}", date);
        assert_eq!(Some(Ok(NaiveDate::from_ymd(2021, 6, 30))), date);
        Ok(())
    }

    #[test]
    fn test_parse_fio_date() -> Result<()> {
        let date = parse_fio_date("30.06.2021");
        println!("date = {:?}", date);
        assert_eq!(Ok(NaiveDate::from_ymd(2021, 6, 30)), date);
        Ok(())
    }
}
