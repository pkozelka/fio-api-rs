
use anyhow::Result; // TODO: use our own error enum

type FioDatum = String;

/// TODO: this must be an enum
type FioFormat = String;

/// TODO: think about better representation, ideally with header+payload structure
type FioOutputStream = String;


pub struct FioClient {
    token: String,
}

impl FioClient {

    pub fn new(token: &str) -> Self {
        Self { token: token.to_string() }
    }

    pub fn get_periods(&self, datum_od: FioDatum, datum_do: FioDatum, format: FioFormat) -> Result<FioOutputStream> {
        todo!()
    }

    pub async fn get_by_id(&self, year: u16, id: u8, format: FioFormat) -> Result<FioOutputStream> {
        // https://www.fio.cz/ib_api/rest/by-id/aGEMtmwcsg5EbfIjqIhunibjhuvfdtsersxexdtgMIdh6u3/2012/1/transactions.cba_xml
        let url = format!("https://www.fio.cz/ib_api/rest/by-id/{token}/{year}/{id}/transactions.{format}",
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

    pub fn get_last(&self, format: FioFormat) -> Result<FioOutputStream> {
        todo!()
    }

    pub fn set_last_id(&self, id: &str) -> Result<()> {
        todo!()
    }

    pub fn set_last_date(&self, date: FioDatum) -> Result<()> {
        todo!()
    }

    pub fn get_merchant(&self, datum_od: FioDatum, datum_do: FioDatum, format: FioFormat) -> Result<FioOutputStream> {
        todo!()
    }

    pub fn get_last_statement(&self) -> Result<String> {
        todo!()
    }

}
