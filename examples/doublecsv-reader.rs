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
            },
            0 => {
                // skip all separator lines
                println!("(-- separator line --)");
                while record.len() == 0 {
                    if !csv_reader.read_record(&mut record)? {
                        break;
                    }
                }
                break;
            },
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
        for (i,heading) in record.iter().enumerate() {
            print!("{}: `{}`, ", heading, data.get(i).unwrap());
        }
        println!()
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use fio_api_rs::csvdata::FioTransactionsRecord;
    use std::io::{Cursor, BufRead};

    #[test]
    fn test_cursor_twoparts() -> anyhow::Result<()> {
        let content = std::fs::read("test/test_by_id.csv").unwrap();
        let mut cursor = Cursor::new(content);

        // info part
        let mut line = String::new();
        while cursor.read_line(&mut line)? > 0 {
            match line.find(';') {
                None => break,
                Some(n) => {
                    let key = &line[0..n];
                    let value = line[n + 1..].trim_end();
                    println!("{}: {}", key, value);
                }
            }
            line.clear();
        }
        println!();

        // data part
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_reader(cursor);
        for r in csv_reader.deserialize() {
            let record: FioTransactionsRecord = r?;
            println!("{:?}", record);
        }
        Ok(())
    }

    #[test]
    fn test_justdata_serde() {
        let mut csv_reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path("test/test_by_id-justdata.csv").unwrap();
        for r in csv_reader.deserialize() {
            let record: FioTransactionsRecord = r.unwrap();
            println!("{:?}", record)
        }
    }
}
