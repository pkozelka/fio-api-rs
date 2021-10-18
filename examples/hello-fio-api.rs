use fio_api::{FioClient, FioResponse};
use fio_api::{FioExportReq, ReportFormat};

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let token = std::fs::read_to_string(".git/fio-test-token")?;
    let fio = FioClient::new(&token);
    let req = FioExportReq::ById {
        year: 2020,
        id: 12,
        format: ReportFormat::Csv,
    };
    let response = fio.export(req)
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
    use std::path::PathBuf;
    use std::str::FromStr;

    use chrono::{Datelike, NaiveDate};

    use fio_api::{DomesticSymbolsBuilder, DomesticTransaction, FioClient, FioClientWithImport, FioExportReq, PaymentBuilder, PaymentType, ReportFormat, TxFormat};

    fn init_logging() {
        std::env::set_var("RUST_LOG", "info,hello_fio_api=debug,fio_api=trace");
        let _ = pretty_env_logger::try_init();
    }

    /// Read FIO token mean for the development testing.
    fn read_devtest_token() -> std::io::Result<(String, PathBuf)> {
        let path = PathBuf::from(".git/fio-test-token");
        let path = path.canonicalize()?;
        let token = std::fs::read_to_string(&path)?;
        Ok((token, path))
    }

    /// Construct simple, read-only client
    fn fio_client() -> FioClient {
        let (token, _) = read_devtest_token().unwrap();
        FioClient::new(&token)
    }

    /// Construct a RW client, taking the account number and currency from the effective filename (behind the symlink)
    fn fio_client_rw() -> FioClientWithImport {
        let (token, path) = read_devtest_token().unwrap();
        let fname = path.file_name().unwrap().to_string_lossy();
        let fnparts: Vec<_> = fname.split(".").collect();
        match fnparts.as_slice() {
            ["fio", account_from, currency, .., "token"] => {
                let fio = FioClient::new(&token);
                FioClientWithImport::new(fio, account_from, currency)
            }
            _ => panic!("Effective filename does not come in form 'fio.<account>.<currency>.WHATEVER.token': '{}'", fname)
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_import_czk_payment() {
        init_logging();
        // curl -S --trace-ascii - -X POST -F "type=xml" -F "token=$(cat .git/fio-test-token)" -F "file=@examples/payment.xml" https://www.fio.cz/ib_api/rest/import/
        let fio = fio_client_rw();
        let payment = fio.new_domestic()
            .amount(321.45)
            .account_to("2702016516", "2010")
            .vs("20")
            .comment("T1")
            .message_for_recipient("t1")
            ;
        let r = fio.import(payment).await.unwrap();
        println!("Payment Response:\nStatus={}\n {:?}", r.status(), r);
        let text = r.text().await.unwrap();
        println!("Response Text: {}", text);
    }

    #[tokio::test]
    #[ignore]
    async fn test_import_czk_two_payments() {
        init_logging();
        let fio = fio_client_rw();
        let payments = vec![
            fio.new_domestic()
                .amount(321.45)
                .account_to("2702016516", "2010")
                .vs("123")
                .comment("T1")
                .message_for_recipient("t1")
                .into(),
            fio.new_domestic()
                .amount(123.45)
                .account_to("2702016516", "2010")
                .vs("1010110101")
                .payment_type(PaymentType::Standard)
                .into(),
        ];
        let r = fio.import(payments.as_slice()).await.unwrap();
        println!("Payment Response:\nStatus={}\n {:?}", r.status(), r);
        let text = r.text().await.unwrap();
        println!("Response Text: {}", text);
    }

    #[tokio::test]
    #[ignore]
    async fn test_by_id() {
        init_logging();
        let fio = fio_client();
        let now = chrono::Utc::now();
        let (year, id): (u16, u8) = if now.month() == 1 {
            ((now.year() - 1) as u16, 12)
        } else {
            (now.year() as u16, (now.month() - 1) as u8)
        };
        let req = FioExportReq::ById { year, id, format: ReportFormat::Csv };
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_periods() {
        init_logging();
        let fio = fio_client();
        let date_start = NaiveDate::from_str("2021-01-01").unwrap();
        let date_end = NaiveDate::from_str("2021-03-31").unwrap();
        let req = FioExportReq::Periods { date_start, date_end, format: TxFormat::Csv };
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_merchant() {
        init_logging();
        let fio = fio_client();
        let date_start = NaiveDate::from_str("2021-01-01").unwrap();
        let date_end = NaiveDate::from_str("2021-06-30").unwrap();
        let req = FioExportReq::Merchant { date_start, date_end, format: TxFormat::Csv };
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_last() {
        init_logging();
        let fio = fio_client();
        let req = FioExportReq::Last { format: TxFormat::Csv };
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_last_statement() {
        init_logging();
        let fio = fio_client();
        let req = FioExportReq::LastStatement;
        let response = fio.export(req).await.unwrap();
        println!("GET {}", response.url());
        println!("HTTP status: {}", response.status());
        let result = response.text().await.unwrap();
        println!("{}", result);
    }
}
