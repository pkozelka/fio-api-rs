use fio_api_rs::FioClient;
use fio_api_rs::export::{FioExportReq, ReportFormat};

#[tokio::main(flavor="current_thread")]
async fn main() {
    let token = std::fs::read_to_string(".git/fio-test-token").unwrap();
    let fio = FioClient::new(&token);
    let response = fio.export(FioExportReq::by_id(2020, 12, ReportFormat::Csv).unwrap())
        .await.unwrap();
    println!("HTTP status: {}", response.status());
    let result = response
        .text().await.unwrap();
    println!("{}", result);
}
