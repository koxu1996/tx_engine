pub mod types;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

use csv::ReaderBuilder;

use crate::core::processor::*;
use types::*;

/// Model that can be used to for processing transactions
/// stored in *.csv* format.
#[derive(Default)]
pub struct CsvProvider {
  processor: BalanceProcessor,
}

impl CsvProvider {
  /// Loads transactions from given path into internal *processor*.
  pub fn load_from_path(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    // Prepare buffered reader.
    let file = File::open(path)?;
    let buffered_file_reader = BufReader::new(file);

    // Delegate parsing to generic *load* method.
    self.load(buffered_file_reader)
  }

  /// Loads transactions from reader into internal *processor*.
  /// Only severe errors are returned; in case of invalid rows, they are
  /// simply ignored and error is logged to stderr.
  pub fn load(&mut self, reader: impl Read) -> Result<(), Box<dyn Error>> {
    // Read CSV data with buffered reader.
    // We use following config:
    // - first row is header,
    // - last columns might be skipped,
    // - trim all whitespaces.
    let mut rdr = ReaderBuilder::new()
      .has_headers(true)
      .flexible(true)
      .trim(csv::Trim::All)
      .from_reader(reader);

    // For every line in CSV perform deserialization into *Row*,
    // then convert it to *Transaction* and finally feed into *processor*.
    // **Note:** We pass transaction ownership to *processor*.
    for result in rdr.deserialize::<TransactionRow>() {
      match result {
        Ok(row) => {
          let tx = row.convert_to_tx();
          match tx {
            Err(e) => {
              eprintln!("Error during convert: {:?}", e);
            }
            Ok(tx) => match self.processor.feed_tx(tx) {
              Ok(_) => {}
              Err(e) => {
                eprintln!("Error during transaction processing: {:?}", e);
              }
            },
          }
        }
        Err(e) => {
          eprintln!("Skipping invalid row: {:?}", e);
        }
      }
    }

    Ok(())
  }

  /// Prints summary (to stdout) of each account in CSV format.
  /// Should be called after *load()* to see results.
  pub fn print_accounts_summary(&self) -> Result<(), Box<dyn Error>> {
    // Create CSV writer for stdout, then go through each account in processor
    // and print its details.
    let mut wtr = csv::Writer::from_writer(io::stdout());
    for (_, account) in self.processor.get_accounts_iter() {
      let acc_row = AccountRow::new(account)?;
      match wtr.serialize(acc_row) {
        Ok(_) => {}
        Err(e) => {
          eprintln!("Unable to serialize account: {:?}", e);
        }
      }
    }

    // Flush stdout.
    wtr.flush()?;

    Ok(())
  }
}
