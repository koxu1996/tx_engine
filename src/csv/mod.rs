pub mod error;
pub mod types;

use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::path::Path;

use csv::ReaderBuilder;

use crate::core::processor::*;
use crate::core::types::Account;
use error::CsvError;
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
  pub fn load_from_path(&mut self, path: impl AsRef<Path>) -> Result<(), CsvError> {
    // Prepare buffered reader. The path goes into the error, so the user
    // learns which file could not be opened.
    let path = path.as_ref();
    let file = File::open(path).map_err(|source| CsvError::OpenFailed {
      path: path.to_path_buf(),
      source,
    })?;
    let buffered_file_reader = BufReader::new(file);

    // Delegate parsing to generic *load* method.
    self.load(buffered_file_reader)
  }

  /// Loads transactions from reader into internal *processor*.
  /// Only severe errors are returned; in case of invalid rows, they are
  /// simply ignored and error is logged to stderr.
  pub fn load(&mut self, reader: impl Read) -> Result<(), CsvError> {
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

    // For every line in CSV perform deserialization into *Transaction*, via *TransactionRow*.
    // **Note:** We pass transaction ownership to *processor*.
    for result in rdr.deserialize::<CsvTransaction>() {
      // Skip invalid rows.
      let Ok(tx) = result.inspect_err(|e| eprintln!("Skipping invalid row: {e}")) else {
        continue;
      };

      // Feed processor with tx.
      if let Err(e) = self.processor.feed_tx(tx.into_inner()) {
        eprintln!("Error during transaction processing: {e}");
      }
    }

    Ok(())
  }

  /// Prints summary (to stdout) of each account in CSV format.
  /// You can choose between sorted or raw order of accounts.
  /// Should be called after *load()* to see results.
  pub fn print_accounts_summary(&self, order: SummaryOrder) -> Result<(), CsvError> {
    // Create CSV writer for stdout.
    let mut wtr = csv::Writer::from_writer(io::stdout());

    // Go through each account in processor and print its details.
    match order {
      SummaryOrder::ByClientId => {
        // We have to collect accounts into sorted vector.
        let mut accounts: Vec<_> = self.processor.get_accounts_iter().collect();
        accounts.sort_unstable_by_key(|a| a.id);
        for account in accounts {
          Self::write_account(&mut wtr, account)?;
        }
      }
      SummaryOrder::Unsorted => {
        for account in self.processor.get_accounts_iter() {
          Self::write_account(&mut wtr, account)?;
        }
      }
    }

    // Flush stdout.
    wtr.flush().map_err(|source| CsvError::WriteFailed { source })?;

    Ok(())
  }

  /// Writes single account as a CSV row.
  /// Serialization error is logged to stderr, but not exits.
  fn write_account(
    wtr: &mut csv::Writer<io::Stdout>,
    account: &Account,
  ) -> Result<(), CsvError> {
    let acc_row = AccountRow::new(account)?;
    if let Err(e) = wtr.serialize(acc_row) {
      eprintln!("Unable to serialize account: {e}");
    }

    Ok(())
  }
}
