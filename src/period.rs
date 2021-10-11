use std::io::{Error, ErrorKind, Result};

use chrono::Datelike;

pub struct FioPeriod {
    pub year: u16,
    // month: 1=Jan, 12=Dec
    pub month: u8,
}

impl FioPeriod {
    pub fn new(year: u16, month: u8) -> Self {
        Self { year, month }
    }

    pub fn current() -> Self {
        let now = chrono::Local::now();
        Self {
            year: now.year() as u16,
            month: now.month() as u8,
        }
    }

    /// parses Period formatted like `2019,10`
    pub fn parse(text: &str) -> Result<FioPeriod> {
        let n = text.find(',').ok_or(Error::new(
            ErrorKind::InvalidData,
            format!("Not a period: '{}'", text),
        ))?;
        let year = &text[..n];
        let year = year.parse().map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Not a year: '{}'; {}", year, e),
            )
        })?;
        let month = &text[n + 1..];
        let month = month.parse().map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("Not a month: '{}'; {}", month, e),
            )
        })?;
        Ok(Self::new(year, month))
    }

    pub fn previous(&self) -> Self {
        if self.month == 1 {
            Self {
                year: self.year - 1,
                month: 12,
            }
        } else {
            Self {
                year: self.year,
                month: self.month - 1,
            }
        }
    }
}

impl ToString for FioPeriod {
    fn to_string(&self) -> String {
        format!("{},{}", self.year, self.month)
    }
}

#[cfg(test)]
mod test {
    use super::FioPeriod;

    #[tokio::test]
    async fn test_parse_period() {
        assert_eq!(
            FioPeriod::parse("2019,3").unwrap().to_string(),
            FioPeriod::new(2019, 3).to_string()
        );
    }

    #[tokio::test]
    async fn test_period_previous() {
        assert_eq!(
            FioPeriod::new(2019, 1).previous().to_string(),
            FioPeriod::new(2018, 12).to_string()
        );
        assert_eq!(
            FioPeriod::new(2019, 2).previous().to_string(),
            FioPeriod::new(2019, 1).to_string()
        );
    }
}
