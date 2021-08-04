//! doc/6: IMPORT (UPLOAD) PLATEBNÍCH PŘÍKAZŮ DO BANK

use reqwest::multipart::Form;
use reqwest::multipart::Part;
use reqwest::Response;

use crate::FioClient;

// TODO
//

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
            .post(format!("{url_base}/import/",
                          url_base = crate::FIOAPI_URL_BASE,
                          // url_base = "http://localhost:1500", // for testing with `nc -l -p 1500`
            ))
            .multipart(form)
            .build()?;
        self.client.execute(http_request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::FioClient;

    /// This test fails.
    /// There are very subtle differences from doing this with CURL (which works fine), like:
    /// - boundary string: curl has shorter, and begins with several dashes
    /// - CURL uses header `Expect: 100-continue`, delaying the body with 1sec
    /// - HTTP headers: reqwest uses lowercase names, and in different order
    /// None of these differences should be significant, but one of them is probably the reason.
    #[tokio::test]
    #[ignore]
    async fn test_import() -> anyhow::Result<()> {
        // curl -S --trace-ascii - -X POST -F "type=xml" -F "token=$(cat ../.git/2702016516-hexakoss-rw.txt)" -F "file=@payment.xml" https://www.fio.cz/ib_api/rest/import/

        std::env::set_var("RUST_LOG", "trace");
        pretty_env_logger::init();

        let token = std::fs::read_to_string(".git/2301479755-OrigisINET-rw.txt")?;
        let payment_xml = std::fs::read_to_string("examples/payment.xml")?;
        let fio = FioClient::new(&token);
        println!("R: {}", payment_xml);
        let response = fio.import(&payment_xml).await?;
        println!("R1:");
        let text = response.text().await?;
        println!("Response: {}", text);
        Ok(())
    }
}
