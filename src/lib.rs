//! FIO API library
//! TODO: use our own error enum

pub use client::FioClient;
pub use error::FioError;
pub use error::Result;
pub use period::FioPeriod;
pub use response::FioResponse;
pub use response::FioResponseInfo;

type FioDatum = String;

pub enum FioRequest {}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(2, 1 + 1, "it works")
    }
}

pub mod csvdata;
pub mod export;
mod import;
mod error;
mod response;
mod tiny_xml;
mod period;
mod client;
