use fio_api_rs::FioClient;

#[tokio::main(flavor="current_thread")]
async fn main() {
    let token = std::fs::read_to_string(".git/fio-test-token").unwrap();
    let fio = FioClient::new(&token);
    let result = fio.get_by_id(2020, 12, "csv".to_string()).await.unwrap();
    println!("{}", result);
}
