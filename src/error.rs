use thiserror::Error as ThisError;
use reqwest::Error;
use serde::Deserialize;

pub type Result<T, E = FioError> = core::result::Result<T, E>;

#[derive(ThisError, Debug)]
pub enum FioError {
    #[error("Failed to prepare HTTP request - check your parameters")]
    ReqwestError(reqwest::Error), // TODO use #[from]

    /// doc/8.1: Pokoušíte se soubor odeslat jako klasický POST a nikoli jako přílohu.
    /// Viz část 6.1 Parametry pro upload dat.
    #[error("The server encoutered an internal error () that prevented it from fulfilling this request.")]
    ServerError,

    /// doc/8.2: Špatně zaslaný dotaz na který server nemůže řádně odpovědět.
    /// Zkontrolujte si parametry URL v dotazu/importu.
    #[error("404 Not Found")]
    BadRequest,

    /// doc/8.3: Není dodržen minimální interval 30 sekund mezi stažením dat z banky / uploadem dat
    /// do banky u konkrétního tokenu (bez ohledu na typ formátu).
    /// Konkrétní token lze použít pouze 1x pro čtení nebo zápis během 30 sekund.
    #[error("409 Conflict")]
    InvalidTiming,

    /// doc/8.4: Chyba indikuje neexistující nebo neaktivní token.
    /// Zkontrolujte si platnost a správnost tokenu v internetovém bankovnictví.
    #[error("500 Internal Server Error")]
    InternalServerError {
        code: u16,
        message: String,
    },

    /// doc/8.5: Při importu příkazů do bankovního systému probíhá kontrola certifikátu certifikační autority.
    /// Tato kontrola selhala a je nutné získat nový používaný certifikát a to buď dle bodu 6.1.,
    /// a nebo přímo ze stránek Fio banky (více viz PDF doc).
    #[error("SSL certificate problem: unable to get local issuer certificate")]
    SslCertProblem,

    /// doc/8.6: Stahujete velkou množinu dat. Limit pro stažení je nastaven na max 50 000 pohybů.
    /// Upravte si adekvátně datumový rozsah v zasílaném dotazu nebo je nutné nastavit zarážku na novější pohyb.
    #[error("413 Příliš mnoho položek")]
    TooManyRows,

    /// (synthetic)
    #[error("Výpis neexistuje")]
    ReportDoesNotExist,

    #[error("Other error, see log for details")]
    OtherError{
        code: String,
        message: String,
    },
}

impl From<reqwest::Error> for FioError {
    fn from(e: Error) -> Self {
        FioError::ReqwestError(e)
    }
}
/*
Sample error XML:
<response xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:noNamespaceSchemaLocation="https://www.fio.cz/schema/response.xsd">
	<result>
		<errorCode>21</errorCode>
		<status>error</status>
		<message>Výpis neexistuje</message>
		<detail></detail>
	</result>
</response>
*/
pub async fn parse_xml_error(response: reqwest::Response) -> FioError {
    match response.headers().get(reqwest::header::CONTENT_TYPE) {
        None => return FioError::OtherError { code: "missing_header".to_string(), message: "content_type".to_string() },
        Some(content_type) => {
            let content_type = content_type.to_str().unwrap_or("(invalid content type)");
            if content_type != "text/xml;charset=UTF-8" {
                return FioError::OtherError { code: "bad_content_type".to_string(), message: format!("{:?}", content_type) }
            }
        },
    }
    let response = response.text().await.unwrap();
    log::trace!("RESPONSE: {}", response);

    let xml: Result<FioResponse, serde_xml_rs::Error> = serde_xml_rs::from_str(&response);
    match xml {
        Ok(xml) => {
            // try to extract more detailed error
            match xml.result.error_code {
                21 => FioError::ReportDoesNotExist,
                _ => FioError::InternalServerError { code: xml.result.error_code, message: xml.result.message }
            }
        },
        Err(e) => {
            FioError::OtherError { code: "xml-error".to_string(), message: format!("{:?}", e) }
        }
    }
}

/// doc/6.1: Schéma XML odpovědi je uvedena na adrese https://www.fio.cz/schema/responseImportIB.xsd
/// TODO: Adjust properly to completely match schema structure
#[derive(Debug, Deserialize, PartialEq)]
struct FioResponse {
    result: FioResponseResult,
}

/// doc/6.1: Odpověď na dávku příkazu je vždy ve formátu XML
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct FioResponseResult {
    /// * `0`:  ok - příkaz byl přijat
    /// * `1`:  nalezené chyby při kontrole příkazů
    /// * `2`:  varování kontrol - chybně vyplněné hodnoty
    /// * `11`: syntaktická chyba
    /// * `12`: prázdný import - v souboru nejsou žádné příkazy
    /// * `13`: příliš dlouhý soubor - soubor je delší než 2 MB
    /// * `14`: prázdný soubor - soubor neobsahuje příkazy
    error_code: u16,
    /// číslo dávky - jednoznačný identifikátor dávky
    id_instruction: Option<String>,
    /// * `ok`: příkaz přijat
    /// * `error`: hrubá chyba v příkazu, dávka se všemi příkazy nebude přijata
    /// * `warning`: varování, některý z údajů nesouhlasí (např. měna platby a měna účtu), příkazy s odpovědí warning byly přijaty bankou
    /// * `fatal`: chyba na straně bankovního systému banky, všechny pokyny se odmítly
    status: String,
    /// suma debetních položek v dávce
    sum_debet: Option<f64>,
    /// suma kreditních položek v dávce
    sum_credit: Option<f64>,
    message: String,
    detail: String,
}
