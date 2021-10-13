//! FIO API library
//! TODO: use our own error enum

pub use client::FioClient;
pub use error::{FioError, Result};
pub use export::{FioExportReq, ReportFormat, TxFormat};
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
