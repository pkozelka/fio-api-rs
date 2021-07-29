use chrono::NaiveDate;

const CSV_HEADER_TRANSACTIONS: &str =
    "ID pohybu;Datum;Objem;Měna;Protiúčet;Název protiúčtu;Kód banky;Název banky;KS;VS;SS;Uživatelská identifikace;Zpráva pro příjemce;Typ;Provedl;Upřesnění;Komentář;BIC;ID pokynu;";
const CSV_HEADER_POS: &str =
    "ID pohybu;ID pokynu;Datum;Objem;Poznámka;Název pobočky;Identifikátor transakce;Číslo zařízení;Datum transakce;Autorizační číslo;Číslo karty;Objem;Měna;Typ;Vystavitel karty;Poplatky celkem;Poplatek Fio;Poplatek intercharge;Poplatek karetní asociace;Zaúčtováno;Datum zaúčtování";

pub struct FioTransactionsRecord {
    id_tx: String,
    date: NaiveDate,
    value: f64,
    currency: String,
    b_account: String,
    b_account_name: String,
    b_bankid: String,
    b_bank_name: String,
    ks: String,
    vs: String,
    ss: String,
    custom_id: String,
    message: String,
    tx_type: TxType,
    who: String,
    note: String,
    comment: String,
    bic: String,
    id_command: String,
}

pub enum TxType {
    // Platba kartou
    card_payment,
    // Příjem převodem uvnitř banky
    fio_income,
    //
    other(String)
}

