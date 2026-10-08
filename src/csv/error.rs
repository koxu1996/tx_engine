use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::core::error::EngineError;

/// Reason why the CSV layer could not read a row or write the summary.
#[derive(Debug, Error)]
pub enum CsvError {
  /// The file with the transactions cannot be opened.
  #[error("cannot open {}: {source}", path.display())]
  OpenFailed {
    path: PathBuf,
    #[source]
    source: io::Error,
  },

  /// A deposit row or a withdrawal row has no amount field.
  #[error("malformed data: deposit and withdrawal must have the amount field")]
  MissingAmount,

  /// A dispute, resolve or chargeback row carries an amount field.
  #[error("malformed data: dispute, resolve and chargeback must not have the amount field")]
  UnexpectedAmount,

  /// The engine refused the transaction that the row describes.
  #[error(transparent)]
  Engine(#[from] EngineError),

  /// A row of the summary cannot be written.
  /// Broken output pipe could trigger this.
  #[error("cannot write the account row: {0}")]
  RowWriteFailed(#[from] csv::Error),

  /// The summary cannot be written to the standard output.
  #[error("cannot write the summary: {source}")]
  WriteFailed {
    #[source]
    source: io::Error,
  },
}
