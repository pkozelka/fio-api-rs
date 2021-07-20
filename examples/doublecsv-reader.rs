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
        for (i,heading) in record.iter().enumerate() {
            print!("{}: `{}`, ", heading, data.get(i).unwrap());
        }
        println!()
    }
    Ok(())
}
