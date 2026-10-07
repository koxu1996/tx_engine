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
    // Extract amount, when the row carries one.
    let amount: Option<Decimal> = match &self.amount {
      None => None,
      Some(s) => match Decimal::from_str(s.as_str()) {
        Ok(v) => Some(v),
        Err(_) => return Err("Unable to parse amount".into()),
      },
    };

    // Map the row onto a transaction. The kinds that move money need
    // the amount, and the other kinds must not carry one, so an invalid
    // pair is rejected here and cannot reach the engine.
    let tx = match (self.type_.as_str(), amount) {
      ("deposit", Some(amount)) => {
        Transaction::Deposit(DepositTx::new(self.client, self.tx, amount)?)
      }
      ("withdrawal", Some(amount)) => {
        Transaction::Withdrawal(WithdrawalTx::new(self.client, self.tx, amount)?)
      }
      ("deposit" | "withdrawal", None) => {
        return Err("Malformed data: deposit/withdrawal must have amount field.".into());
      }
      ("dispute", None) => Transaction::Dispute(DisputeTx {
        client: self.client,
        ref_tx: self.tx,
      }),
      ("resolve", None) => Transaction::Resolve(ResolveTx {
        client: self.client,
        ref_tx: self.tx,
      }),
      ("chargeback", None) => Transaction::Chargeback(ChargebackTx {
        client: self.client,
        ref_tx: self.tx,
      }),
      ("dispute" | "resolve" | "chargeback", Some(_)) => {
        return Err("Malformed data: dispute/resolve/chargeback cannot have amount field.".into());
      }
      _ => return Err("Invalid transaction type".into()),
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
      locked: account.is_locked,
    })
  }
}
