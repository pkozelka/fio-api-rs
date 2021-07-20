//! doc/5: EXPORT (DOWNLOAD) POHYBŮ A VÝPISŮ Z BANKY
//!
use anyhow::Result;
use strum_macros::IntoStaticStr;

use crate::{FIOAPI_URL_BASE, FioClient, FioDatum};
use reqwest::Response;

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

pub struct FioExportReq {
    command: &'static str,
    params: String,
}

impl FioExportReq {
    /// doc/5.2.1: Pohyby na účtu za určené období
    /// * `date_start`: datum - začátek stahovaných příkazů ve formátu rok-měsíc-den (rrrr-mm-dd)
    /// * `date_end`: datum - konec stahovaných příkazů ve formátu rok-měsíc-den (rrrr-mm-dd)
    /// * `format`: formát pohybů
    pub fn periods(date_start: FioDatum, date_end: FioDatum, format: ReportFormat) -> Result<Self> {
        let format: &'static str = format.into();
        let params = format!("{datum_od}/{datum_do}/transactions.{format}",
                             datum_od = date_start,
                             datum_do = date_end,
                             format = format);
        Ok(Self { command: "periods", params })
    }

    /// doc/5.2.2: Oficiální výpisy pohybů z účtu
    /// * `format`: formát pohybů
    pub fn by_id(year: u16, id: u8, format: ReportFormat) -> Result<Self> {
        let format: &'static str = format.into();
        let params = format!("{year}/{id}/transactions.{format}",
                             year = year,
                             id = id,
                             format = format);
        Ok(Self { command: "by-id", params })
    }

    /// doc/5.2.3: Pohyby na účtu od posledního stažení
    /// * `format`: formát pohybů
    pub fn last(format: ReportFormat) -> Result<Self> {
        let format: &'static str = format.into();
        let params = format!("transactions.{format}",
                             format = format);
        Ok(Self { command: "last", params })
    }

    /// doc/5.2.4: Nastavení zarážky
    /// 1) Na ID posledního úspěšně staženého pohybu
    /// * `id`: ID posledního úspěšně staženého pohybu
    pub fn set_last_id(id: &str) -> Result<Self> {
        let params = id.to_string();
        Ok(Self { command: "set-last-id", params })
    }

    /// doc/5.2.4: Nastavení zarážky
    /// 2) Na datum posledního neúspěšně staženého dne
    /// * `date`: datum poslední neúspěšně staženého výpisu ve formátu rok- měsíc- den (rrrr-mm-dd)
    pub fn set_last_date(date: FioDatum) -> Result<Self> {
        let params = date;
        Ok(Self { command: "set-last-date", params })
    }

    /// doc/5.2.5: Karetní transakce obchodníka za určené období
    /// * `date_start`: datum - začátek stahovaných příkazů ve formátu rok-měsíc-den (rrrr-mm-dd)
    /// * `date_end`: datum - konec stahovaných příkazů ve formátu rok-měsíc-den (rrrr-mm-dd)
    /// * `format`: formát pohybů
    pub fn merchant(date_start: FioDatum, date_end: FioDatum, format: ReportFormat) -> Result<Self> {
        let format: &'static str = format.into();
        let params = format!("{datum_od}/{datum_do}/transactions.{format}",
                             datum_od = date_start,
                             datum_do = date_end,
                             format = format);
        Ok(Self { command: "merchant", params })
    }

    /// doc/5.2.6: Číslo posledního vytvořeného oficiálního výpisu
    pub fn last_statement() -> Result<Self> {
        Ok(Self { command: "lastStatement", params: "".to_string()})
    }

    fn build_url(&self, token: &str) -> String {
        format!("{url_base}/{command}/{token}/{params}",
                          url_base = FIOAPI_URL_BASE,
                          command = self.command,
                          token = token,
                          params = self.params)
    }
}

impl FioClient {
    pub async fn export(&self, fio_req: FioExportReq) -> Result<Response> {
        let http_request = self.client
            .get(fio_req.build_url(&self.token))
            .build()?;
        Ok(self.client.execute(http_request).await?)
    }
}
