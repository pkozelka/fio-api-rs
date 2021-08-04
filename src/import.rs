//! doc/6: IMPORT (UPLOAD) PLATEBNÍCH PŘÍKAZŮ DO BANK

use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::Response;

use crate::FioClient;

// TODO
//

impl FioClient {
    pub async fn import(&self) -> reqwest::Result<Response> {
        // doc/6.1
        let payment_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<Import xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://www.fio.cz/schema/importIB.xsd">
    <Orders>
        <DomesticTransaction>
            <accountFrom>2702016516</accountFrom>
            <currency>CZK</currency>
            <amount>100.00</amount>
            <accountTo>2301479755</accountTo>
            <bankCode>2010</bankCode>
            <date>2021-08-04</date>
        </DomesticTransaction>
    </Orders>
</Import>
"#;
        let part = Part::bytes(payment_xml.as_bytes())
            .file_name("payment-orders.txt")
            .mime_str("text/xml")?;
        let form = Form::new()
            .text("token", self.token.to_string())
            .text("type", "xml")
            .part("file", part)
            .text("lng", "en");
        let http_request = self.client
            .post(format!("{url_base}/import/", url_base = crate::FIOAPI_URL_BASE))
            .multipart(form)
            .build()?;

        self.client.execute(http_request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::FioClient;

    #[tokio::test]
    #[ignore]
    async fn test_import() -> anyhow::Result<()> {
        // curl -S --trace-ascii - -X POST -F "type=xml" -F "token=$(cat ../.git/2702016516-hexakoss-rw.txt)" -F "file=@payment.xml" https://www.fio.cz/ib_api/rest/import/

        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::init();

        let token = std::fs::read_to_string(".git/2702016516-hexakoss-rw.txt").unwrap();
        let fio = FioClient::new(&token);
        println!("R:");
        let response = fio.import().await?;
        println!("R1:");
        let text = response.text().await?;
        println!("Response: {}", text);
        Ok(())
    }
}
