pub mod types;

use std::error::Error;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

use csv::ReaderBuilder;

use crate::core::processor::*;
use crate::core::types::Account;
use types::*;

/// Order of the accounts in the printed summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SummaryOrder {
  ByClientId,
  Unsorted,
}

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
  /// You can choose between sorted or raw order of accounts.
  /// Should be called after *load()* to see results.
  pub fn print_accounts_summary(&self, order: SummaryOrder) -> Result<(), Box<dyn Error>> {
    // Create CSV writer for stdout.
    let mut wtr = csv::Writer::from_writer(io::stdout());

    // Go through each account in processor and print its details.
    match order {
      SummaryOrder::ByClientId => {
        // We have to collect accounts into sorted vector.
        let mut accounts: Vec<_> = self.processor.get_accounts_iter().map(|(_, a)| a).collect();
        accounts.sort_unstable_by_key(|a| a.id);
        for account in accounts {
          Self::write_account(&mut wtr, account)?;
        }
      }
      SummaryOrder::Unsorted => {
        for (_, account) in self.processor.get_accounts_iter() {
          Self::write_account(&mut wtr, account)?;
        }
      }
    }

    // Flush stdout.
    wtr.flush()?;

    Ok(())
  }

  /// Writes single account as a CSV row.
  /// Serialization error is logged to stderr, but not exits.
  fn write_account(
    wtr: &mut csv::Writer<io::Stdout>,
    account: &Account,
  ) -> Result<(), Box<dyn Error>> {
    let acc_row = AccountRow::new(account)?;
    if let Err(e) = wtr.serialize(acc_row) {
      eprintln!("Unable to serialize account: {:?}", e);
    }

    Ok(())
  }
}
