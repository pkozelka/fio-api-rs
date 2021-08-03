use fio_api_rs::{FioClient, FioResponse};
use fio_api_rs::export::{FioExportReq, ReportFormat};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let token = std::fs::read_to_string(".git/fio-test-token")?;
    let fio = FioClient::new(&token);
    let response = fio.export(FioExportReq::by_id(2020, 12, ReportFormat::Csv)?)
        .await?;
    println!("HTTP status: {}", response.status());
    let mut response = FioResponse::try_from(response).await?;
    let info = response.info()?;
    println!("Account number: {}/{} ({})", info.account_id()?, info.bank_id()?, info.currency()?);
    println!("IBAN / BIC: {} / {}", info.iban()?, info.bic()?);
    println!("ID: {} .. {}", info.id_from()?, info.id_to()?);
    println!("Date: {} .. {}", info.date_start()?, info.date_end()?);
    println!("-- DATA --");
    for record in response.data()? {
        let record = record?;
        println!("{:?}", record);
    }
    println!("Balance: {} .. {}", info.opening_balance()?, info.closing_balance()?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::NaiveDate;

    use fio_api_rs::export::{FioExportReq, ReportFormat, TxFormat};
    use fio_api_rs::FioClient;

    fn fio_client() -> FioClient {
        std::env::set_var("RUST_LOG", "fio_api_rs=trace");
        pretty_env_logger::init();
        let token = std::fs::read_to_string(".git/fio-test-token").unwrap();
        FioClient::new(&token)
    }

    #[tokio::test]
    async fn test_by_id() {
        let fio = fio_client();
        let req = FioExportReq::by_id(2021, 11, ReportFormat::Csv).unwrap();
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    async fn test_periods() {
        let fio = fio_client();
        let date_start = NaiveDate::from_str("2021-01-01").unwrap();
        let date_end = NaiveDate::from_str("2021-03-31").unwrap();
        let req = FioExportReq::periods(date_start, date_end, TxFormat::Csv).unwrap();
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    async fn test_merchant() {
        let fio = fio_client();
        let date_start = NaiveDate::from_str("2021-01-01").unwrap();
        let date_end = NaiveDate::from_str("2021-06-30").unwrap();
        let req = FioExportReq::merchant(date_start, date_end, TxFormat::Csv).unwrap();
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    async fn test_last() {
        let fio = fio_client();
        let req = FioExportReq::last(TxFormat::Csv).unwrap();
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    async fn test_last_statement() {
        let fio = fio_client();
        let req = FioExportReq::last_statement().unwrap();
        let response = fio.export(req).await.unwrap();
        println!("GET {}", response.url());
        println!("HTTP status: {}", response.status());
        let result = response.text().await.unwrap();
        println!("{}", result);
    }
}
