use fio_api_rs::FioClient;
use fio_api_rs::export::{FioExportReq, ReportFormat};

#[tokio::main(flavor="current_thread")]
async fn main() {
    let token = std::fs::read_to_string(".git/fio-test-token").unwrap();
    let fio = FioClient::new(&token);
    let response = fio.export(FioExportReq::by_id(2020, 12, ReportFormat::Csv).unwrap())
        .await.unwrap();
    println!("HTTP status: {}", response.status());
    let result = response.text().await.unwrap();
    println!("{}", result);
}

#[cfg(test)]
mod tests {
    use fio_api_rs::FioClient;
    use fio_api_rs::export::{FioExportReq, ReportFormat, TxFormat};
    use chrono::NaiveDate;
    use std::str::FromStr;

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
