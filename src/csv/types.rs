use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;

use crate::core::error::EngineError;
use crate::core::types::{
  Account, ChargebackTx, ClientId, DepositTx, DisputeTx, ResolveTx, Transaction, TransactionId,
  TxAmount, WithdrawalTx,
};
use crate::csv::error::CsvError;

/// Transaction type, as it appears in the *type* column.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RowType {
  Deposit,
  Withdrawal,
  Dispute,
  Resolve,
  Chargeback,
}

#[derive(Debug, Deserialize)]
/// Model that represents CSV row with transaction details.
pub struct TransactionRow {
  #[serde(rename = "type")]
  type_: RowType,
  client: ClientId,
  tx: TransactionId,
  #[serde(default, with = "rust_decimal::serde::str_option")]
  amount: Option<Decimal>,
}

/// A *Transaction* as it arrives from a CSV row.
#[derive(Debug, Deserialize)]
#[serde(try_from = "TransactionRow")]
pub struct CsvTransaction(Transaction);

impl CsvTransaction {
  /// The transaction that the row described.
  pub fn into_inner(self) -> Transaction {
    self.0
  }
}

impl TryFrom<TransactionRow> for CsvTransaction {
  type Error = CsvError;

  /// Converts row into valid *Transaction*, which is returned afterwards.
  /// In case of any problem Error is returned.
  fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
    use Transaction as T;

    // Map amount field into a TxAmount, if present.
    let amount = row.amount.map(TxAmount::new).transpose()?;

    // Map the row onto a transaction.
    let tx = match (row.type_, amount) {
      (RowType::Deposit, Some(amount)) => T::Deposit(DepositTx::new(row.client, row.tx, amount)),
      (RowType::Withdrawal, Some(amount)) => {
        T::Withdrawal(WithdrawalTx::new(row.client, row.tx, amount))
      }
      (RowType::Deposit | RowType::Withdrawal, None) => {
        return Err(CsvError::MissingAmount);
      }
      (RowType::Dispute, None) => T::Dispute(DisputeTx {
        client: row.client,
        ref_tx: row.tx,
      }),
      (RowType::Resolve, None) => T::Resolve(ResolveTx {
        client: row.client,
        ref_tx: row.tx,
      }),
      (RowType::Chargeback, None) => T::Chargeback(ChargebackTx {
        client: row.client,
        ref_tx: row.tx,
      }),
      (RowType::Dispute | RowType::Resolve | RowType::Chargeback, Some(_)) => {
        return Err(CsvError::UnexpectedAmount);
      }
    };

    Ok(Self(tx))
  }
}

#[derive(Debug, Serialize)]
/// Model that represents CSV row with account details.
pub struct AccountRow {
  client: ClientId,
  /// For all three numbers we rely on rust_decimal feature to avoid
  /// default float variant that drops digits during serialization.
  #[serde(with = "rust_decimal::serde::str")]
  available: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  held: Decimal,
  #[serde(with = "rust_decimal::serde::str")]
  total: Decimal,
  locked: bool,
}

impl AccountRow {
  /// Creates CSV row from existing *Account*.
  pub fn new(account: &Account) -> Result<Self, EngineError> {
    Ok(Self {
      client: account.id(),
      available: account.amount_available(),
      held: account.amount_held(),
      total: account.amount_total()?,
      locked: account.is_locked(),
    })
  }
}
