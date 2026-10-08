use rust_decimal::Decimal;

use crate::core::error::EngineError;

/// Client ID
pub type ClientId = u16;

/// Structure that holds client accouts details.
#[derive(Debug)]
pub struct Account {
  /// Client ID.
  id: ClientId,
  /// Available amount.
  amount_available: Decimal,
  /// Held amount (under dispute).
  amount_held: Decimal,
  /// Is account locked?
  is_locked: bool,
}

impl Account {
  /// Constructs new account.
  pub fn new(id: ClientId) -> Self {
    Self {
      id,
      amount_available: Decimal::ZERO,
      amount_held: Decimal::ZERO,
      is_locked: false,
    }
  }

  /// Client ID of the account.
  pub fn id(&self) -> ClientId {
    self.id
  }

  /// Amount that the client can use.
  pub fn amount_available(&self) -> Decimal {
    self.amount_available
  }

  /// Amount that a dispute holds.
  pub fn amount_held(&self) -> Decimal {
    self.amount_held
  }

  /// Is account locked?
  pub fn is_locked(&self) -> bool {
    self.is_locked
  }

  /// Locks the account, as a result of a chargeback.
  /// The lock is final, because only a human can release it, so there is
  /// no function that unlocks the account.
  pub fn lock(&mut self) {
    self.is_locked = true;
  }

  /// Calculates total amount of money,
  /// which is sum of available and held.
  /// Returns error when the sum does not fit in **Decimal**.
  pub fn amount_total(&self) -> Result<Decimal, EngineError> {
    self
      .amount_available
      .checked_add(self.amount_held)
      .ok_or(EngineError::TotalOverflowed { client: self.id })
  }

  /// Increases available amount, as a result of deposit.
  /// Returns error when the result does not fit in **Decimal**.
  pub fn deposit(&mut self, amount: TxAmount) -> Result<(), EngineError> {
    let available = self
      .amount_available
      .checked_add(amount.get())
      .ok_or(EngineError::AvailableOverflowed { client: self.id })?;

    // Extra check: total must stay inside a **Decimal** as well.
    available
      .checked_add(self.amount_held)
      .ok_or(EngineError::TotalOverflowed { client: self.id })?;

    self.amount_available = available;

    Ok(())
  }

  /// Decreases available amount, as a result of withdrawal.
  /// Returns error when the result does not fit in **Decimal**.
  pub fn withdraw(&mut self, amount: TxAmount) -> Result<(), EngineError> {
    self.amount_available = self
      .amount_available
      .checked_sub(amount.get())
      .ok_or(EngineError::AvailableOverflowed { client: self.id })?;

    Ok(())
  }

  /// Moves amount from available to held, as a result of dispute.
  /// Both amounts change together, or neither of them changes.
  pub fn hold(&mut self, amount: TxAmount) -> Result<(), EngineError> {
    // Calculate both amounts first.
    let available = self
      .amount_available
      .checked_sub(amount.get())
      .ok_or(EngineError::AvailableOverflowed { client: self.id })?;
    let held = self
      .amount_held
      .checked_add(amount.get())
      .ok_or(EngineError::HeldOverflowed { client: self.id })?;

    self.amount_available = available;
    self.amount_held = held;

    Ok(())
  }

  /// Moves amount from held back to available, as a result of resolve.
  /// Both amounts change together, or neither of them changes.
  pub fn release(&mut self, amount: TxAmount) -> Result<(), EngineError> {
    let available = self
      .amount_available
      .checked_add(amount.get())
      .ok_or(EngineError::AvailableOverflowed { client: self.id })?;
    let held = self
      .amount_held
      .checked_sub(amount.get())
      .ok_or(EngineError::HeldOverflowed { client: self.id })?;

    self.amount_available = available;
    self.amount_held = held;

    Ok(())
  }

  /// Decreases held amount, as a result of chargeback.
  /// The money leaves the account, so it does not return to available.
  pub fn withdraw_held(&mut self, amount: TxAmount) -> Result<(), EngineError> {
    self.amount_held = self
      .amount_held
      .checked_sub(amount.get())
      .ok_or(EngineError::HeldOverflowed { client: self.id })?;

    Ok(())
  }
}

/// Dispute status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisputeStatus {
  Started,
  Resolved,
  ChargedBack,
}

/// Transaction ID
pub type TransactionId = u32;

/// Amount that transaction moves, always above zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxAmount(Decimal); // use Decimal to avoid round-off

impl TxAmount {
  /// Constructs new *TxAmount*.
  /// Returns error when the value is not above zero.
  pub fn new(value: Decimal) -> Result<Self, EngineError> {
    if value <= Decimal::ZERO {
      return Err(EngineError::AmountNotPositive);
    }

    Ok(Self(value))
  }

  /// The value as a **Decimal**.
  pub fn get(self) -> Decimal {
    self.0
  }
}

/// Details of deposit transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DepositTx {
  /// Client ID.
  pub client: ClientId,
  /// Transaction ID.
  pub tx: TransactionId,
  /// Deposited amount.
  amount: TxAmount,
}

impl DepositTx {
  /// Constructs new *DepositTx*.
  pub fn new(client: ClientId, tx: TransactionId, amount: TxAmount) -> Self {
    Self { client, tx, amount }
  }

  /// Deposited amount.
  pub fn amount(&self) -> TxAmount {
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
  amount: TxAmount,
}

impl WithdrawalTx {
  /// Constructs new *WithdrawalTx*.
  pub fn new(client: ClientId, tx: TransactionId, amount: TxAmount) -> Self {
    Self { client, tx, amount }
  }

  /// Withdrawn amount.
  pub fn amount(&self) -> TxAmount {
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

/// A transaction that the engine stored, together with the state of its
/// dispute.
///
/// Only these two kinds are ever stored, and only a deposit can be
/// disputed, so a dispute of a withdrawal has no representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoredTx {
  Deposit {
    deposit: DepositTx,
    dispute: Option<DisputeStatus>,
  },
  Withdrawal(WithdrawalTx),
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
