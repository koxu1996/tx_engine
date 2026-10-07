use std::error::Error;

use rust_decimal::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::core::types::*;

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

impl TryFrom<TransactionRow> for Transaction {
  type Error = Box<dyn Error>;

  /// Converts row into valid *Transaction*, which is returned afterwards.
  /// In case of any problem Error is returned.
  fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
    let amount = row.amount;

    // Map the row onto a transaction. The kinds that move money need
    // the amount, and the other kinds must not carry one, so an invalid
    // pair is rejected here and cannot reach the engine.
    let tx = match (row.type_, amount) {
      (RowType::Deposit, Some(amount)) => {
        Self::Deposit(DepositTx::new(row.client, row.tx, TxAmount::new(amount)?))
      }
      (RowType::Withdrawal, Some(amount)) => Self::Withdrawal(WithdrawalTx::new(
        row.client,
        row.tx,
        TxAmount::new(amount)?,
      )),
      (RowType::Deposit | RowType::Withdrawal, None) => {
        return Err("Malformed data: deposit/withdrawal must have amount field.".into());
      }
      (RowType::Dispute, None) => Self::Dispute(DisputeTx {
        client: row.client,
        ref_tx: row.tx,
      }),
      (RowType::Resolve, None) => Self::Resolve(ResolveTx {
        client: row.client,
        ref_tx: row.tx,
      }),
      (RowType::Chargeback, None) => Self::Chargeback(ChargebackTx {
        client: row.client,
        ref_tx: row.tx,
      }),
      (RowType::Dispute | RowType::Resolve | RowType::Chargeback, Some(_)) => {
        return Err("Malformed data: dispute/resolve/chargeback cannot have amount field.".into());
      }
    };

    Ok(tx)
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
  /// Creates CSV row from existing *Account*. No error is expected here.
  pub fn new(account: &Account) -> Result<Self, Box<dyn Error>> {
    Ok(Self {
      client: account.id,
      available: account.amount_available,
      held: account.amount_held,
      total: account.amount_total()?,
      locked: account.is_locked(),
    })
  }
}
