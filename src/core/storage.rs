use std::collections::HashMap;
use std::error::Error;

use crate::core::types::*;

/// Common storage for accounts, transactions and their disputes.
#[derive(Default)]
pub struct Storage {
  pub accounts: AccountsStorage,
  pub transactions: TransactionsStorage,
  pub disputes: DisputesStorage,
}

/// Map holding client Id and corresponding *Account*.
#[derive(Default)]
pub struct AccountsStorage(HashMap<ClientId, Account>);

impl AccountsStorage {
  /// Gets account from storage, when the client is already known.
  pub fn get(&self, id: &ClientId) -> Option<&Account> {
    self.0.get(id)
  }

  /// Gets account from storage - mutable version.
  pub fn get_mut(&mut self, id: &ClientId) -> Result<&mut Account, Box<dyn Error>> {
    self.0.get_mut(id).ok_or_else(|| "Account not found".into())
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

/// Map holding transaction Id and corresponding *Transaction*.
#[derive(Default)]
pub struct TransactionsStorage(HashMap<TransactionId, Transaction>);

impl TransactionsStorage {
  /// Adds deposit to storage, under its own transaction ID.
  /// *Caution:* make sure transaction id is unique, otherwise
  /// existing data will be overwritten.
  pub fn add_deposit(&mut self, deposit: DepositTx) {
    self.0.insert(deposit.tx, Transaction::Deposit(deposit));
  }

  /// Adds withdrawal to storage, under its own transaction ID.
  /// *Caution:* make sure transaction id is unique, otherwise
  /// existing data will be overwritten.
  pub fn add_withdrawal(&mut self, withdrawal: WithdrawalTx) {
    self
      .0
      .insert(withdrawal.tx, Transaction::Withdrawal(withdrawal));
  }

  /// Gets transaction from storage.
  pub fn get(&self, id: &TransactionId) -> Result<&Transaction, Box<dyn Error>> {
    self.0.get(id).ok_or_else(|| "Transaction not found".into())
  }

  /// Checks if given ID is unique among already existing transactions
  pub fn assert_unique_id(&self, id: &TransactionId) -> Result<(), Box<dyn Error>> {
    if self.0.contains_key(id) {
      return Err("Transaction ID is not unique".into());
    }
    Ok(())
  }
}

/// Map holding transaction Id and corresponding *DisputeStatus*.
#[derive(Default)]
pub struct DisputesStorage(HashMap<TransactionId, DisputeStatus>);

impl DisputesStorage {
  /// Opens new dispute for given transaction.
  /// *Caution:* make sure transaction id is unique, otherwise
  /// existing data will be overwritten.
  pub fn open(&mut self, id: TransactionId) {
    self.0.insert(id, DisputeStatus::Started);
  }

  /// Checks if dispute for given transaction is already stored.
  pub fn has(&self, id: &TransactionId) -> bool {
    self.0.contains_key(id)
  }

  /// Gets dispute from storage - mutable version.
  pub fn get_mut(&mut self, id: &TransactionId) -> Result<&mut DisputeStatus, Box<dyn Error>> {
    self.0.get_mut(id).ok_or_else(|| "Dispute not found".into())
  }
}
