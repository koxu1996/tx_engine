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
      amount_available: Decimal::ZERO,
      amount_held: Decimal::ZERO,
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

/// Validates the amount of a transaction that moves money.
fn validate_amount(amount: Decimal) -> Result<(), Box<dyn Error>> {
  if amount <= Decimal::ZERO {
    return Err("Amount must be greater than 0".into());
  }

  Ok(())
}

/// Details of deposit transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepositTx {
  /// Client ID.
  pub client: ClientId,
  /// Transaction ID.
  pub tx: TransactionId,
  /// Deposited amount.
  amount: Decimal, // use Decimal to avoid round-off
}

impl DepositTx {
  /// Constructs new *DepositTx*.
  /// Returns error when the amount is not above zero.
  pub fn new(client: ClientId, tx: TransactionId, amount: Decimal) -> Result<Self, Box<dyn Error>> {
    validate_amount(amount)?;

    Ok(Self { client, tx, amount })
  }

  /// Deposited amount, which is always above zero.
  pub fn amount(&self) -> Decimal {
    self.amount
  }
}

/// Details of withdrawal transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WithdrawalTx {
  /// Client ID.
  pub client: ClientId,
  /// Transaction ID.
  pub tx: TransactionId,
  /// Withdrawn amount.
  amount: Decimal, // use Decimal to avoid round-off
}

impl WithdrawalTx {
  /// Constructs new *WithdrawalTx*.
  /// Returns error when the amount is not above zero.
  pub fn new(client: ClientId, tx: TransactionId, amount: Decimal) -> Result<Self, Box<dyn Error>> {
    validate_amount(amount)?;

    Ok(Self { client, tx, amount })
  }

  /// Withdrawn amount, which is always above zero.
  pub fn amount(&self) -> Decimal {
    self.amount
  }
}

/// Details of dispute transaction.
/// It refers to an earlier transaction, so it carries no amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisputeTx {
  /// Client ID.
  pub client: ClientId,
  /// ID of the referenced transaction.
  pub ref_tx: TransactionId,
}

/// Details of resolve transaction.
/// It refers to an earlier transaction, so it carries no amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolveTx {
  /// Client ID.
  pub client: ClientId,
  /// ID of the referenced transaction.
  pub ref_tx: TransactionId,
}

/// Details of chargeback transaction.
/// It refers to an earlier transaction, so it carries no amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChargebackTx {
  /// Client ID.
  pub client: ClientId,
  /// ID of the referenced transaction.
  pub ref_tx: TransactionId,
}

/// Transaction - deposit / withdrawal / dispute, etc.
///
/// Each kind carries its own details. Only the kinds that move money
/// carry an amount, so a deposit without an amount, and a dispute with
/// one, have no representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transaction {
  Deposit(DepositTx),
  Withdrawal(WithdrawalTx),
  Dispute(DisputeTx),
  Resolve(ResolveTx),
  Chargeback(ChargebackTx),
}

impl Transaction {
  /// Client ID of this transaction.
  pub fn client(&self) -> ClientId {
    match self {
      Transaction::Deposit(details) => details.client,
      Transaction::Withdrawal(details) => details.client,
      Transaction::Dispute(details) => details.client,
      Transaction::Resolve(details) => details.client,
      Transaction::Chargeback(details) => details.client,
    }
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
