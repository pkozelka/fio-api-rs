
use anyhow::Result; // TODO: use our own error enum
use strum_macros::IntoStaticStr;

type FioDatum = String;

/// 5.1 Supported transaction formats
#[derive(IntoStaticStr)]
pub enum TxFormat {
    #[strum(serialize = "csv")]
    Csv,
    #[strum(serialize = "gpc")]
    Gpc,
    #[strum(serialize = "html")]
    Html,
    #[strum(serialize = "json")]
    Json,
    #[strum(serialize = "ofx")]
    Ofx,
    #[strum(serialize = "xml")]
    FioXml,
}

/// 5.1 Supported report formats
#[derive(IntoStaticStr)]
pub enum ReportFormat {
    #[strum(serialize = "csv")]
    Csv,
    #[strum(serialize = "gpc")]
    Gpc,
    #[strum(serialize = "html")]
    Html,
    #[strum(serialize = "json")]
    Json,
    #[strum(serialize = "ofx")]
    Ofx,
    #[strum(serialize = "xml")]
    FioXml,
    #[strum(serialize = "pdf")]
    Pdf,
    #[strum(serialize = "sta")]
    Mt940,
    /// `CAMT.053`
    #[strum(serialize = "cba_xml")]
    CbaXml,
    /// `CAMT.053`
    #[strum(serialize = "sba_xml")]
    SbaXml,
}

/// TODO: think about better representation, ideally with header+payload structure
type FioOutputStream = String;

const URL_PREFIX: &'static str = "https://www.fio.cz/ib_api/rest";

pub struct FioClient {
    token: String,
}

impl FioClient {

    pub fn new(token: &str) -> Self {
        Self { token: token.to_string() }
    }

    pub fn get_periods(&self, datum_od: FioDatum, datum_do: FioDatum, format: TxFormat) -> Result<FioOutputStream> {
        todo!()
    }

    pub async fn get_by_id(&self, year: u16, id: u8, format: ReportFormat) -> Result<FioOutputStream> {
        let format: &'static str = format.into();
        let url = format!("{url_prefix}/by-id/{token}/{year}/{id}/transactions.{format}",
            url_prefix = URL_PREFIX,
            token = self.token,
            year = year,
            id = id,
            format = format
        );
        // TODO: use shared client, to minimize handshake overhead
        // TODO: share the "get" core
        Ok(reqwest::get(url).await?
            .text().await?)
    }

    pub fn get_last(&self, format: TxFormat) -> Result<FioOutputStream> {
        todo!()
    }

    pub fn set_last_id(&self, id: &str) -> Result<()> {
        todo!()
    }

    pub fn set_last_date(&self, date: FioDatum) -> Result<()> {
        todo!()
    }

    pub fn get_merchant(&self, datum_od: FioDatum, datum_do: FioDatum, format: ReportFormat) -> Result<FioOutputStream> {
        todo!()
    }

    pub fn get_last_statement(&self) -> Result<String> {
        todo!()
    }

}
