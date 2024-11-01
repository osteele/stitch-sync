use csv::ReaderBuilder;
use std::io::Cursor;

pub struct CsvReader {
    headers: csv::StringRecord,
    records: csv::StringRecordsIntoIter<Cursor<String>>,
}

impl CsvReader {
    pub fn from_str(csv_data: &str) -> Result<Self, csv::Error> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(Cursor::new(csv_data.to_string()));

        let headers = reader.headers()?.clone();
        let records = reader.into_records();

        Ok(Self { headers, records })
    }

    pub fn iter_records(&mut self) -> CsvRecordsIterator<'_> {
        CsvRecordsIterator {
            headers: &self.headers,
            records: &mut self.records,
        }
    }
}

pub struct CsvRecordsIterator<'a> {
    headers: &'a csv::StringRecord,
    records: &'a mut csv::StringRecordsIntoIter<Cursor<String>>,
}

impl<'a> Iterator for CsvRecordsIterator<'a> {
    type Item = Result<CsvRecord<'a>, csv::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.records.next().map(|result| {
            result.map(|record| CsvRecord {
                record,
                headers: self.headers,
            })
        })
    }
}

pub struct CsvRecord<'a> {
    record: csv::StringRecord,
    headers: &'a csv::StringRecord,
}

impl<'a> CsvRecord<'a> {
    pub fn get(&self, column_name: &str) -> Option<&str> {
        let idx = self.headers.iter().position(|h| h == column_name)?;
        self.record.get(idx)
    }

    pub fn get_vec(&self, column_name: &str, separator: char) -> Option<Vec<String>> {
        self.get(column_name)
            .filter(|value| !value.is_empty())
            .map(|value| {
                value
                    .split(separator)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
    }
}
