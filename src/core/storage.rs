use std::collections::HashMap;

use crate::core::error::EngineError;
use crate::core::types::{Account, ClientId, DepositTx, StoredTx, TransactionId, WithdrawalTx};

/// Common storage for accounts and transactions.
#[derive(Default)]
pub struct Storage {
  pub accounts: AccountsStorage,
  pub transactions: TransactionsStorage,
}

/// Map holding client Id and corresponding *Account*.
#[derive(Default)]
pub struct AccountsStorage(HashMap<ClientId, Account>);

impl AccountsStorage {
  /// Gets account from storage, when the client is already known.
  pub fn get(&self, id: ClientId) -> Option<&Account> {
    self.0.get(&id)
  }

  /// Gets account from storage - mutable version.
  pub fn get_mut(&mut self, id: ClientId) -> Result<&mut Account, EngineError> {
    self
      .0
      .get_mut(&id)
      .ok_or(EngineError::AccountNotFound { client: id })
  }

  /// Gets account from storage, or creates an empty one when the client
  /// is not known yet.
  pub fn get_or_create(&mut self, id: ClientId) -> &mut Account {
    self.0.entry(id).or_insert_with(|| Account::new(id))
  }

  /// Gets iterator over the stored accounts.
  pub fn iter(&self) -> impl Iterator<Item = &Account> {
    self.0.values()
  }
}

/// Map holding transaction Id and corresponding *StoredTx*.
#[derive(Default)]
pub struct TransactionsStorage(HashMap<TransactionId, StoredTx>);

impl TransactionsStorage {
  /// Adds deposit to storage, under its own transaction ID.
  /// *Caution:* make sure transaction id is unique, otherwise
  /// existing data will be overwritten.
  pub fn add_deposit(&mut self, deposit: DepositTx) {
    self.0.insert(
      deposit.tx,
      StoredTx::Deposit {
        deposit,
        dispute: None,
      },
    );
  }

  /// Adds withdrawal to storage, under its own transaction ID.
  /// *Caution:* make sure transaction id is unique, otherwise
  /// existing data will be overwritten.
  pub fn add_withdrawal(&mut self, withdrawal: WithdrawalTx) {
    self
      .0
      .insert(withdrawal.tx, StoredTx::Withdrawal(withdrawal));
  }

  /// Gets stored transaction from storage - mutable version.
  /// The dispute state lives on the stored deposit, so a caller that
  /// changes it needs this handle.
  pub fn get_mut(&mut self, id: TransactionId) -> Result<&mut StoredTx, EngineError> {
    self
      .0
      .get_mut(&id)
      .ok_or(EngineError::TransactionNotFound { tx: id })
  }

  /// Checks if a transaction with given ID is already stored.
  pub fn has(&self, id: TransactionId) -> bool {
    self.0.contains_key(&id)
  }
}
