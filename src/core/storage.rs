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
  /// Adds client account to storage.
  /// *Caution:* make sure account id is unique, otherwise
  /// existing data will be overwritten.
  pub fn add(&mut self, account: Account) {
    let id = account.id;
    self.0.insert(id, account);
  }

  /// Checks if given client is already stored.
  pub fn has(&self, id: &ClientId) -> bool {
    self.0.contains_key(id)
  }

  /// Gets account from storage - mutable version.
  pub fn get_mut(&mut self, id: &ClientId) -> Result<&mut Account, Box<dyn Error>> {
    let account = match self.0.get_mut(id) {
      None => return Err("Account not found".into()),
      Some(x) => x,
    };
    Ok(account)
  }

  /// Removes account from storage.
  pub fn remove(&mut self, id: &ClientId) {
    self.0.remove(id);
  }

  /// Gets raw map used by storage.
  pub fn raw(&self) -> &HashMap<ClientId, Account> {
    &self.0
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
    let tx = match self.0.get(id) {
      None => return Err("Transaction not found".into()),
      Some(x) => x,
    };
    Ok(tx)
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
  /// Adds dispute to storage.
  /// *Caution:* make sure transaction id is unique, otherwise
  /// existing data will be overwritten.
  pub fn add(&mut self, id: TransactionId, dispute: DisputeStatus) {
    self.0.insert(id, dispute);
  }

  /// Gets dispute from storage.
  pub fn get(&self, id: &TransactionId) -> Result<&DisputeStatus, Box<dyn Error>> {
    let dispute = match self.0.get(id) {
      None => return Err("Dispute not found".into()),
      Some(x) => x,
    };
    Ok(dispute)
  }

  /// Gets transaction from storage - mutable version.
  pub fn get_mut(&mut self, id: &TransactionId) -> Result<&mut DisputeStatus, Box<dyn Error>> {
    let dispute = match self.0.get_mut(id) {
      None => return Err("Dispute not found".into()),
      Some(x) => x,
    };
    Ok(dispute)
  }
}
