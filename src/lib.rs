//! FIO API library

pub use client::{FioClient, FioClientWithImport};
pub use error::{FioError, Result};
pub use export::{FioExportReq, ReportFormat, TxFormat};
pub use import::{DomesticPayment, DomesticSymbolsBuilder, DomesticTransaction, ForeignPayment, ForeignTransaction, PaymentBuilder, T2Payment, T2Transaction};
pub use period::FioPeriod;
pub use response::{FioResponse, FioResponseInfo};

type FioDatum = String;

pub enum FioRequest {}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(2, 1 + 1, "it works")
    }
}

mod csvdata;
mod export;
mod import;
mod error;
mod response;
mod tiny_xml;
mod period;
mod client;
