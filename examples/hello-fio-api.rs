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

    use fio_api::{FioClient, FioExportReq, Payment, ReportFormat, TxFormat};

    fn fio_client_rw() -> (FioClient, String, String) {
        std::env::set_var("RUST_LOG", "trace, fio_api=trace");
        pretty_env_logger::init();
        let path = PathBuf::from_str(".git/fio-test-token").unwrap();
        let path = path.canonicalize().unwrap();
        let path = path.as_path();
        let fname = path.file_name().unwrap().to_string_lossy();
        let fnparts: Vec<_> = fname.split(".").collect();
        let token = std::fs::read_to_string(path).unwrap();
        match fnparts.as_slice() {
            ["fio", acnt, curr, ..] => {
                let fio = FioClient::new(&token);
                (fio, acnt.to_string(), curr.to_string())
            },
            _ => panic!("Effective filename does not come in form 'fio.<account>.<currency>.WHATEVER': '{}'", fname)
        }
    }

    fn fio_client() -> FioClient {
        let (fio, _, _) = fio_client_rw();
        fio
    }

    #[tokio::test]
    #[ignore]
    async fn test_import_czk_payment() {
        // curl -S --trace-ascii - -X POST -F "type=xml" -F "token=$(cat .git/fio-test-token)" -F "file=@examples/payment.xml" https://www.fio.cz/ib_api/rest/import/
        let (fio, account_from, currency) = fio_client_rw();
        let payment = Payment::default()
            .date(chrono::Local::now().date().naive_local())
            .account_from(account_from)
            .currency(currency)
            .amount(321.45)
            .account_to("2702016516")
            .vs("20")
            .bank_code("2010")
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
    async fn test_by_id() {
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
        let fio = fio_client();
        let req = FioExportReq::Last { format: TxFormat::Csv };
        let response = fio.export(req).await.unwrap();
        let result = response.text().await.unwrap();
        println!("{}", result);
    }

    #[tokio::test]
    #[ignore]
    async fn test_last_statement() {
        let fio = fio_client();
        let req = FioExportReq::LastStatement;
        let response = fio.export(req).await.unwrap();
        println!("GET {}", response.url());
        println!("HTTP status: {}", response.status());
        let result = response.text().await.unwrap();
        println!("{}", result);
    }
}
