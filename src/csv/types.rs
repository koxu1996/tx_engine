use std::error::Error;

use rust_decimal::prelude::*;
use serde::Deserialize;
use serde::Serialize;

use crate::core::types::*;

#[derive(Debug, Deserialize)]
/// Model that represents CSV row with transaction details.
pub struct TransactionRow {
  #[serde(rename = "type")]
  type_: String,
  client: u16,
  tx: u32,
  amount: Option<String>,
}

impl TransactionRow {
  /// Converts row into valid *Transaction*, which is returned afterwards.
  /// In case of any problem Error is returned.
  pub fn convert_to_tx(&self) -> Result<Transaction, Box<dyn Error>> {
    // Validate transaction type.
    let type_ = match self.type_.as_str() {
      "deposit" => TransactionType::Deposit,
      "withdrawal" => TransactionType::Withdrawal,
      "dispute" => TransactionType::Dispute,
      "resolve" => TransactionType::Resolve,
      "chargeback" => TransactionType::Chargeback,
      _ => return Err("Invalid transaction type".into()),
    };

    // Extract amount.
    let amount: Option<Decimal> = match &self.amount {
      None => None,
      Some(s) => {
        let val = Decimal::from_str(s.as_str());
        match val {
          Ok(v) => Some(v),
          Err(_) => {
            return Err("Unable to parse amount".into());
          }
        }
      }
    };

    // Construct new transaction from row details.
    let tx = Transaction::new(type_, self.client, self.tx, amount)?;

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
      total: (account.amount_available + account.amount_held),
      locked: account.is_locked,
    })
  }
}
