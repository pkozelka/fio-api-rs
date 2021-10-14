//! doc/5: EXPORT (DOWNLOAD) POHYBŮ A VÝPISŮ Z BANKY
//!
use std::fmt::{Display, Formatter};

use chrono::NaiveDate;
use strum_macros::IntoStaticStr;

use crate::FioDatum;

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
#[derive(IntoStaticStr, Copy, Clone)]
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

#[derive(IntoStaticStr)]
pub enum FioExportReq {
    /// doc/5.2.1: Pohyby na účtu za určené období
    #[strum(serialize = "periods")]
    Periods {
        /// začátek stahovaných příkazů ve formátu rok-měsíc-den (rrrr-mm-dd)
        date_start: NaiveDate,
        /// konec stahovaných příkazů ve formátu rok-měsíc-den (rrrr-mm-dd)
        date_end: NaiveDate,
        /// formát pohybů
        format: TxFormat,
    },
    /// doc/5.2.2: Oficiální výpisy pohybů z účtu
    #[strum(serialize = "by-id")]
    ById {
        year: u16,
        id: u8,
        /// formát pohybů
        format: ReportFormat,
    },
    /// doc/5.2.3: Pohyby na účtu od posledního stažení
    #[strum(serialize = "last")]
    Last {
        /// formát pohybů
        format: TxFormat
    },
    /// doc/5.2.4: Nastavení zarážky
    /// 1) Na ID posledního úspěšně staženého pohybu
    #[strum(serialize = "set-last-id")]
    SetLastId {
        /// ID posledního úspěšně staženého pohybu
        id: String
    },
    /// doc/5.2.4: Nastavení zarážky
    /// 2) Na datum posledního neúspěšně staženého dne
    #[strum(serialize = "set-last-date")]
    SetLastDate {
        /// datum poslední neúspěšně staženého výpisu ve formátu rok- měsíc- den (rrrr-mm-dd)
        date: FioDatum
    },
    /// doc/5.2.5: Karetní transakce obchodníka za určené období
    #[strum(serialize = "merchant")]
    Merchant {
        /// začátek stahovaných příkazů ve formátu rok-měsíc-den (rrrr-mm-dd)
        date_start: NaiveDate,
        /// konec stahovaných příkazů ve formátu rok-měsíc-den (rrrr-mm-dd)
        date_end: NaiveDate,
        /// formát pohybů
        format: TxFormat,
    },
    /// doc/5.2.6: Číslo posledního vytvořeného oficiálního výpisu
    #[strum(serialize = "lastStatement")]
    LastStatement,
}

impl FioExportReq {
    pub(crate) fn build_url(&self, token: &str) -> String {
        let command: &'static str = self.into();
        let params = match self {
            FioExportReq::Periods { date_start, date_end, format } =>
                format!("{datum_od}/{datum_do}/transactions.{format}",
                        datum_od = date_start,
                        datum_do = date_end,
                        format = Into::<&'static str>::into(format)),
            FioExportReq::ById { year, id, format } =>
                format!("{year}/{id}/transactions.{format}",
                        year = year,
                        id = id,
                        format = Into::<&'static str>::into(format)),
            FioExportReq::Last { format } =>
                format!("transactions.{format}",
                        format = Into::<&'static str>::into(format)),
            FioExportReq::SetLastId { id } =>
                id.to_string(),
            FioExportReq::SetLastDate { date } =>
                date.to_string(),
            FioExportReq::Merchant { date_start, date_end, format } =>
                format!("{datum_od}/{datum_do}/transactions.{format}",
                        datum_od = date_start,
                        datum_do = date_end,
                        format = Into::<&'static str>::into(format)),
            FioExportReq::LastStatement =>
                "statement".to_string(),
        };
        format!("{url_base}/{command}/{token}/{params}",
                url_base = crate::client::FIOAPI_URL_BASE,
                command = command,
                token = token,
                params = params)
    }
}

impl Display for ReportFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.into())
    }
}
