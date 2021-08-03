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

/// Translation of CSV response.
/// The text will be typically received from calls to [Fio REST API](crate::export::FioExportReq):
/// - [by_id](crate::export::FioExportReq::by_id)
/// - [periods](crate::export::FioExportReq::periods)
/// - [merchant](crate::export::FioExportReq::merchant)
///
/// Note that this object wraps a cursor, and therefore reads the data in the order in which they come.
///
/// It is expected to be used like this:
///
/// ```no_run
/// use fio_api_rs::FioResponse;
///
/// let http_response = ...;
/// let mut response = FioResponse::from(http_response);
/// let info = response.info()?; // optional; get initial info so that it can be used after iteration
/// let data = response.data()?; // get the iterator over records; consumes the response!
/// // iterate over the rows
/// for record in data {
///     let record = record?;
///     println!("{:?}", record);
/// }
/// // work with the info, which was read earlier
/// println!("Balance: {} .. {}", info.opening_balance()?, info.closing_balance()?);
/// ```
pub struct FioResponse {
    cursor: Cursor<Vec<u8>>,
}

impl From<Cursor<Vec<u8>>> for FioResponse {
    /// Wrap cursor as a [`FioResponse`].
    /// This can be used for reading from a file or in-memory string.
    fn from(cursor: Cursor<Vec<u8>>) -> Self {
        Self { cursor }
    }
}

impl FioResponse {
    /// Try to process a response from executing a [crate::export::FioExportReq] with [reqwest::Client].
    pub async fn try_from(response: Response) -> crate::Result<Self> {
        // analyze HTTP headers
        match response.status().as_u16() {
            200..=299 => {
                let bytes = response.bytes().await?.to_vec();
                let cursor = Cursor::new(bytes);
                Ok(Self::from(cursor))
            }
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

    pub fn data(self) -> std::io::Result<DeserializeRecordsIntoIter<Cursor<Vec<u8>>, FioTransactionsRecord>> {
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

const INFO_ACCOUNT_ID: &'static str = "accountId";
const INFO_BANK_ID: &'static str = "bankId";
const INFO_CURRENCY: &'static str = "currency";
const INFO_IBAN: &'static str = "iban";
const INFO_BIC: &'static str = "bic";
const INFO_OPENING_BALANCE: &'static str = "openingBalance";
const INFO_CLOSING_BALANCE: &'static str = "closingBalance";
const INFO_DATE_START: &'static str = "dateStart";
const INFO_DATE_END: &'static str = "dateEnd";
const INFO_ID_FROM: &'static str = "idFrom";
const INFO_ID_TO: &'static str = "idTo";

const UNICODE_BOM: char = '\u{feff}';

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
            if line.starts_with(UNICODE_BOM) {
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

    fn get_info(&self, key: &str) -> crate::Result<&str> {
        match self.info_headers.get(key) {
            None => Err(crate::error::FioError::MissingInfoField(key.to_string())),
            Some(s) => Ok(s.as_str())
        }
    }

    /// Consumes instance, returning internal representation of the response info.
    pub fn into_inner(self) -> HashMap<String, String> {
        self.info_headers
    }

    /// Returns reference to internal representation of the response info.
    pub fn get_ref(&self) -> &HashMap<String, String> {
        &self.info_headers
    }

    pub fn account_id(&self) -> crate::Result<&str> {
        self.get_info(INFO_ACCOUNT_ID)
    }

    pub fn bank_id(&self) -> crate::Result<&str> {
        self.get_info(INFO_BANK_ID)
    }

    pub fn currency(&self) -> crate::Result<&str> {
        self.get_info(INFO_CURRENCY)
    }

    pub fn iban(&self) -> crate::Result<&str> {
        self.get_info(INFO_IBAN)
    }

    pub fn bic(&self) -> crate::Result<&str> {
        self.get_info(INFO_BIC)
    }

    pub fn opening_balance(&self) -> crate::Result<f64> {
        let s = self.get_info(INFO_OPENING_BALANCE)?;
        parse_fio_decimal(s)
            .map_err(crate::error::FioError::from)
    }

    pub fn closing_balance(&self) -> crate::Result<f64> {
        let s = self.get_info(INFO_CLOSING_BALANCE)?;
        parse_fio_decimal(s)
            .map_err(crate::error::FioError::from)
    }

    pub fn date_start(&self) -> crate::Result<NaiveDate> {
        let s = self.get_info(INFO_DATE_START)?;
        parse_fio_date(s)
            .map_err(crate::error::FioError::from)
    }

    pub fn date_end(&self) -> crate::Result<NaiveDate> {
        let s = self.get_info(INFO_DATE_END)?;
        parse_fio_date(s)
            .map_err(crate::error::FioError::from)
    }

    pub fn id_from(&self) -> crate::Result<&str> {
        self.get_info(INFO_ID_FROM)
    }

    pub fn id_to(&self) -> crate::Result<&str> {
        self.get_info(INFO_ID_TO)
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
        let balance = info.opening_balance()?;
        println!("balance = {:?}", balance);
        assert_eq!(4789.51_f64, balance);
        Ok(())
    }

    #[test]
    fn test_parse_date() -> Result<()> {
        let info = FioResponseInfo::sample1()?;
        let date = info.date_end()?;
        println!("date = {:?}", date);
        assert_eq!(NaiveDate::from_ymd(2021, 6, 30), date);
        Ok(())
    }

    #[test]
    fn test_parse_fio_date() -> Result<()> {
        let date = parse_fio_date("30.06.2021")?;
        println!("date = {:?}", date);
        assert_eq!(NaiveDate::from_ymd(2021, 6, 30), date);
        Ok(())
    }
}
