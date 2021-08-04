//! doc/6: IMPORT (UPLOAD) PLATEBNÍCH PŘÍKAZŮ DO BANK

use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::Response;

use crate::FioClient;

// TODO: support all payment types
// TODO: define strict formats for payments
// TODO: enhance error xml to receive all fields

impl FioClient {
    pub async fn import(&self, payment_xml: &str) -> reqwest::Result<Response> {
        // doc/6.1
        let part = Part::text(payment_xml.to_string())
            .file_name("payments.xml")
            .mime_str("application/xml")?;
        let form = Form::new()
            .text("type", "xml")
            .text("token", self.token.to_string())
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

        let token = std::fs::read_to_string(".git/2301479755-OrigisINET-rw.txt")?;
        let payment_xml = std::fs::read_to_string("examples/payment.xml")?;
        let fio = FioClient::new(token.trim());
        println!("R: {}", payment_xml);
        let response = fio.import(&payment_xml).await?;
        println!("R1:");
        let text = response.text().await?;
        println!("Response: {}", text);
        Ok(())
    }
}

// Response: <?xml version="1.0" encoding="UTF-8" standalone="yes"?><responseImport xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="http://www.fio.cz/schema/responseImport.xsd"><result><errorCode>0</errorCode><idInstruction>1788006308</idInstruction><status>ok</status><sums><sum id="CZK"><sumCredit>0</sumCredit><sumDebet>102.93</sumDebet></sum></sums></result><ordersDetails><detail id="1"><messages><message status="ok" errorCode="0">OK</message></messages></detail></ordersDetails></responseImport>
