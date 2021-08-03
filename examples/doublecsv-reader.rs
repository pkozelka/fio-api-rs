use csv::StringRecord;

fn main() -> std::io::Result<()> {
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .flexible(true)
        .from_path("test/test_by_id.csv").unwrap();
    let mut record = StringRecord::new();
    while csv_reader.read_record(&mut record)? {
        match record.len() {
            2 => {
                // the heading part
                let key = record.get(0).unwrap();
                let value = record.get(1).unwrap();
                println!("{} = {}", key, value);
            }
            0 => {
                // skip all separator lines
                println!("(-- separator line --)");
                while record.len() == 0 {
                    if !csv_reader.read_record(&mut record)? {
                        break;
                    }
                }
                break;
            }
            _ => {
                println!("Column count: {}", record.len());
                break;
            }
        }
    }
    println!("(-- headers --)");
    println!("{:?}", record);
    println!("(-- data --)");
    for r in csv_reader.records() {
        let data = r?;
        if data.len() != record.len() {
            panic!("Column count differs: data={} headers={}", data.len(), record.len());
        }
        for (i, heading) in record.iter().enumerate() {
            print!("{}: `{}`, ", heading, data.get(i).unwrap());
        }
        println!()
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use fio_api_rs::{FioResponse, FioResponseInfo};
    use fio_api_rs::csvdata::FioTransactionsRecord;

    #[test]
    fn test_cursor_twoparts() -> anyhow::Result<()> {
        let content = std::fs::read("test/test_by_id.csv")?;
        let mut cursor = Cursor::new(content);

        let info = FioResponseInfo::read(&mut cursor)?;
        println!("{:?} :: {:?}", info.iban(), info.bic());
        // data part
        let response = FioResponse::from(cursor);
        for r in response.data()? {
            let record: FioTransactionsRecord = r?;
            println!("{:?}", record);
        }
        Ok(())
    }

    #[test]
    fn test_cursor_onlydata() -> anyhow::Result<()> {
        let content = std::fs::read("test/test_by_id.csv")?;
        let cursor = Cursor::new(content);

        // data part
        let response = FioResponse::from(cursor);
        for r in response.data()? {
            let record: FioTransactionsRecord = r?;
            println!("{:?}", record);
        }
        Ok(())
    }
}
