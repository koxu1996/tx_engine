use std::error::Error;

use rust_decimal::prelude::*;

/// Client ID
pub type ClientId = u16;

#[derive(Debug)]
/// Structure that holds client accouts details.
pub struct Account {
  /// Client ID.
  pub id: ClientId,
  /// Available amount.
  pub amount_available: Decimal,
  /// Held amount (under dispute).
  pub amount_held: Decimal,
  /// Is account locked?
  pub is_locked: bool,
  _private: (),
}

impl Account {
  /// Constructs new account.
  pub fn new(id: ClientId) -> Self {
    Account {
      id,
      amount_available: Decimal::new(0, 0),
      amount_held: Decimal::new(0, 0),
      is_locked: false,
      _private: (),
    }
  }

  /// Calculates total amount of money,
  /// which is sum of available and held.
  /// Returns error when the sum does not fit in **Decimal**.
  pub fn amount_total(&self) -> Result<Decimal, Box<dyn Error>> {
    self
      .amount_available
      .checked_add(self.amount_held)
      .ok_or_else(|| "Total amount overflowed".into())
  }
}

#[derive(Debug, PartialEq, Eq)]
/// Dispute status
pub enum DisputeStatus {
  Started,
  Resolved,
  Chargeback,
}

/// Transaction ID
pub type TransactionId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Transaction type - deposit / withdrawal / dispute, etc.
pub enum TransactionType {
  Deposit,
  Withdrawal,
  Dispute,
  Resolve,
  Chargeback,
}

#[derive(Debug)]
pub struct Transaction {
  /// Type: deposit / withdrawal / dispute, etc.
  pub kind: TransactionType,
  /// Client ID.
  pub client: ClientId,
  /// Transaction ID.
  pub tx: TransactionId,
  /// Associated amount.
  pub amount: Option<Decimal>, // use Decimal to avoid round-off
  _private: (),
}

impl Transaction {
  /// Constructs new *Transaction*.
  pub fn new(
    kind: TransactionType,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Decimal>,
  ) -> Result<Self, Box<dyn Error>> {
    // Validate amount: must be above 0.0.
    if let Some(x) = amount {
      if x <= Decimal::ZERO {
        return Err("Amount must be greater than 0".into());
      }
    }

    // Reject malformed data: when dispute/resolve/chargeback has amount field.
    if matches!(
      kind,
      TransactionType::Dispute | TransactionType::Resolve | TransactionType::Chargeback
    ) && amount.is_some()
    {
      return Err("Malformed data: dispute/resolve/chargeback cannot have amount field.".into());
    }

    Ok(Self {
      kind,
      client,
      tx,
      amount,
      _private: (),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// The sum of two maximal amounts does not fit in a **Decimal**.
  /// The function must return an error, and must not panic.
  #[test]
  fn amount_total_returns_error_on_overflow() {
    let mut account = Account::new(1);
    account.amount_available = Decimal::MAX;
    account.amount_held = Decimal::MAX;

    assert!(account.amount_total().is_err());
  }
}
