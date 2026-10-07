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
  client: u16,
  tx: u32,
  amount: Option<String>,
}

impl TryFrom<TransactionRow> for Transaction {
  type Error = Box<dyn Error>;

  /// Converts row into valid *Transaction*, which is returned afterwards.
  /// In case of any problem Error is returned.
  fn try_from(row: TransactionRow) -> Result<Self, Self::Error> {
    // Extract amount, when the row carries one.
    let amount = row
      .amount
      .as_deref()
      .map(Decimal::from_str)
      .transpose()
      .map_err(|_| "Unable to parse amount")?;

    // Map the row onto a transaction. The kinds that move money need
    // the amount, and the other kinds must not carry one, so an invalid
    // pair is rejected here and cannot reach the engine.
    let tx = match (row.type_, amount) {
      (RowType::Deposit, Some(amount)) => {
        Self::Deposit(DepositTx::new(row.client, row.tx, amount)?)
      }
      (RowType::Withdrawal, Some(amount)) => {
        Self::Withdrawal(WithdrawalTx::new(row.client, row.tx, amount)?)
      }
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
  client: u16,
  available: Decimal,
  held: Decimal,
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
