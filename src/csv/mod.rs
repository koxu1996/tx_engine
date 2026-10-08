pub mod error;
pub mod types;

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use csv::ReaderBuilder;

use crate::core::processor::BalanceProcessor;
use crate::core::types::Account;
use error::CsvError;
use types::{AccountRow, CsvTransaction};

/// Reports how many rows were accepted and how many were skipped.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LoadReport {
  pub accepted: usize,
  pub skipped: usize,
}

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
  pub fn load_from_path(&mut self, path: impl AsRef<Path>) -> Result<LoadReport, CsvError> {
    // Open the file. The path goes into the error, so the user learns
    // which file could not be opened.
    // **Note:** the CSV reader buffers on its own, so the file needs no
    // *BufReader* around it.
    let path = path.as_ref();
    let file = File::open(path).map_err(|source| CsvError::OpenFailed {
      path: path.to_path_buf(),
      source,
    })?;

    // Delegate parsing to generic *load* method.
    self.load(file)
  }

  /// Loads transactions from reader into internal *processor*, and reports
  /// how many rows it accepted and skipped.
  /// An invalid row never stops the load; it is logged to stderr instead.
  /// A failure of the reader itself does stop it, because no further row
  /// can arrive.
  pub fn load(&mut self, reader: impl Read) -> Result<LoadReport, CsvError> {
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

    // Read the header here - we can catch invalid file or broken reader early.
    rdr.headers()?;

    // For every line in CSV perform deserialization into *Transaction*, via *TransactionRow*.
    // **Note:** We pass transaction ownership to *processor*.
    let mut report = LoadReport::default();
    for result in rdr.deserialize::<CsvTransaction>() {
      // Skip invalid rows, but stop when the reader itself failed.
      let tx = match result {
        Ok(tx) => tx,
        Err(e) if matches!(e.kind(), csv::ErrorKind::Io(_)) => return Err(e.into()),
        Err(e) => {
          eprintln!("Skipping invalid row: {e}");
          report.skipped += 1;
          continue;
        }
      };

      // Feed processor with tx.
      match self.processor.feed_tx(tx.into_inner()) {
        Ok(()) => report.accepted += 1,
        Err(e) => {
          report.skipped += 1;
          eprintln!("Error during transaction processing: {e}");
        }
      }
    }

    Ok(report)
  }

  /// Writes summary of each account in CSV format into given *writer*.
  /// You can choose between sorted or raw order of accounts.
  /// Should be called after *load()* to see results.
  pub fn write_accounts_summary(
    &self,
    order: SummaryOrder,
    writer: impl Write,
  ) -> Result<(), CsvError> {
    // Create CSV writer over the given sink.
    let mut wtr = csv::Writer::from_writer(writer);

    // Go through each account in processor and print its details.
    match order {
      SummaryOrder::ByClientId => {
        // We have to collect accounts into sorted vector.
        let mut accounts: Vec<_> = self.processor.accounts().collect();
        accounts.sort_unstable_by_key(|a| a.id());
        for account in accounts {
          Self::write_account(&mut wtr, account)?;
        }
      }
      SummaryOrder::Unsorted => {
        for account in self.processor.accounts() {
          Self::write_account(&mut wtr, account)?;
        }
      }
    }

    // Flush stdout.
    wtr
      .flush()
      .map_err(|source| CsvError::WriteFailed { source })?;

    Ok(())
  }

  /// Writes single account as a CSV row.
  /// A failure stops the summary, because it means the sink is gone.
  fn write_account<W: Write>(wtr: &mut csv::Writer<W>, account: &Account) -> Result<(), CsvError> {
    let acc_row = AccountRow::new(account)?;
    wtr.serialize(acc_row).map_err(CsvError::RowWriteFailed)?;

    Ok(())
  }
}
