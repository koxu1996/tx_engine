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

  /// Increases available amount, as a result of deposit.
  /// Returns error when the result does not fit in **Decimal**.
  pub fn deposit(&mut self, amount: Decimal) -> Result<(), Box<dyn Error>> {
    self.amount_available = self
      .amount_available
      .checked_add(amount)
      .ok_or_else(|| Box::<dyn Error>::from("Available amount overflowed"))?;

    Ok(())
  }

  /// Decreases available amount, as a result of withdrawal.
  /// Returns error when the result does not fit in **Decimal**.
  pub fn withdraw(&mut self, amount: Decimal) -> Result<(), Box<dyn Error>> {
    self.amount_available = self
      .amount_available
      .checked_sub(amount)
      .ok_or_else(|| Box::<dyn Error>::from("Available amount overflowed"))?;

    Ok(())
  }

  /// Moves amount from available to held, as a result of dispute.
  /// Both amounts change together, or neither of them changes.
  pub fn hold(&mut self, amount: Decimal) -> Result<(), Box<dyn Error>> {
    // Calculate both amounts first.
    let available = self
      .amount_available
      .checked_sub(amount)
      .ok_or_else(|| Box::<dyn Error>::from("Available amount overflowed"))?;
    let held = self
      .amount_held
      .checked_add(amount)
      .ok_or_else(|| Box::<dyn Error>::from("Held amount overflowed"))?;

    self.amount_available = available;
    self.amount_held = held;

    Ok(())
  }

  /// Moves amount from held back to available, as a result of resolve.
  /// Both amounts change together, or neither of them changes.
  pub fn release(&mut self, amount: Decimal) -> Result<(), Box<dyn Error>> {
    let available = self
      .amount_available
      .checked_add(amount)
      .ok_or_else(|| Box::<dyn Error>::from("Available amount overflowed"))?;
    let held = self
      .amount_held
      .checked_sub(amount)
      .ok_or_else(|| Box::<dyn Error>::from("Held amount overflowed"))?;

    self.amount_available = available;
    self.amount_held = held;

    Ok(())
  }

  /// Decreases held amount, as a result of chargeback.
  /// The money leaves the account, so it does not return to available.
  pub fn withdraw_held(&mut self, amount: Decimal) -> Result<(), Box<dyn Error>> {
    self.amount_held = self
      .amount_held
      .checked_sub(amount)
      .ok_or_else(|| Box::<dyn Error>::from("Held amount overflowed"))?;

    Ok(())
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
