use fio_api_rs::{FioClient, ReportFormat};

#[tokio::main(flavor="current_thread")]
async fn main() {
    let token = std::fs::read_to_string(".git/fio-test-token").unwrap();
    let fio = FioClient::new(&token);
    let result = fio.get_by_id(2020, 12, ReportFormat::Csv).await.unwrap();
    println!("{}", result);
}
