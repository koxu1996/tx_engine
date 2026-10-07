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
      id: id,
      amount_available: Decimal::new(0, 0),
      amount_held: Decimal::new(0, 0),
      is_locked: false,
      _private: (),
    }
  }

  /// Calculates total amount of money,
  /// which is sum of available and held.
  pub fn amount_total(&self) -> Decimal {
    let total = self.amount_available + self.amount_held;
    total
  }
}

#[derive(Debug, PartialEq, Eq)]
/// Dispute status
pub enum DisputeStatus {
  Started,
  Resolved,
  Chargeback,
}

#[derive(Debug)]
pub struct Dispute {
  /// Current status of dispute
  status: DisputeStatus,
  _private: (),
}

impl Dispute {
  /// Constructs new *Dispute*.
  pub fn new() -> Self {
    Self {
      status: DisputeStatus::Started,
      _private: (),
    }
  }

  /// Updates dispute status to **Resolved**.
  pub fn mark_resolved(&mut self) {
    self.status = DisputeStatus::Resolved;
  }

  /// Updates dispute status to **Chargeback**.
  pub fn mark_chargeback(&mut self) {
    self.status = DisputeStatus::Resolved;
  }

  /// Getter for status field.
  pub fn get_status(&self) -> &DisputeStatus {
    &self.status
  }
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
      if x < Decimal::new(0, 0) {
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
