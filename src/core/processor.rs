use std::collections::hash_map;
use std::error::Error;

use crate::core::storage::*;
use crate::core::types::*;

/// Balance processor, with internal storage for accounts/transactions/disputes.
#[derive(Default)]
pub struct BalanceProcessor {
  storage: Storage,
}

impl BalanceProcessor {
  /// Processes deposit transaction.
  /// Validation rules:
  /// 1. Transaction ID is unique.
  /// 2. Client account exists.
  /// 3. Account is not locked.
  /// Effects:
  /// 1. Client's available amount is increased.
  /// 2. Transaction is stored.
  fn process_deposit(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
    // Validation
    self.storage.transactions.assert_unique_id(&tx.tx)?;
    let account = self.storage.accounts.get_mut(&tx.client)?;
    if account.is_locked {
      return Err("Account is locked!".into());
    }
    let amount = match tx.amount {
      None => return Err("Deposit transaction must have amount!".into()),
      Some(x) => x,
    };

    // Effects
    account.amount_available += amount;
    self.storage.transactions.add(tx);

    Ok(())
  }

  /// Processes withdrawal transaction.
  /// Validation rules:
  /// 1. Transaction ID is unique.
  /// 2. Client account exists.
  /// 3. Account is not locked.
  /// 4. Client has enough money available.
  /// Effects:
  /// 1. Client's available amount is decreased.
  /// 2. Transaction is stored.
  fn process_withdrawal(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
    // Validation
    self.storage.transactions.assert_unique_id(&tx.tx)?;
    let account = self.storage.accounts.get_mut(&tx.client)?;
    if account.is_locked {
      return Err("Account is locked!".into());
    }
    let amount = match tx.amount {
      None => return Err("Deposit transaction must have amount!".into()),
      Some(x) => x,
    };
    if account.amount_available < amount {
      return Err("Not sufficient funds to make withdrawal".into());
    }

    // Effects
    account.amount_available -= amount;
    self.storage.transactions.add(tx);

    Ok(())
  }

  /// Processes dispute transaction.
  /// Validation rules:
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction has matching client ID.
  /// 3. Referenced transaction is deposit.
  /// 4. Referenced transaction has amount associated.
  /// 5. Referenced transaction is not already disputed.
  /// 6. Client account exists.
  /// Effects:
  /// 1. Client's available amount is decreased.
  /// 2. Client's held amount is increased.
  /// 3. Dispute is stored.
  fn process_dispute(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
    // Validation
    let ref_tx = self.storage.transactions.get(&tx.tx)?;
    if ref_tx.client != tx.client {
      return Err("Mismatched client id!".into());
    }
    if ref_tx.kind != TransactionType::Deposit {
      return Err("Cannot dispute transaction different than deposit".into());
    }
    let ref_tx_amount = match ref_tx.amount {
      Some(x) => x,
      None => return Err("Missing associated amount!".into()),
    };
    if let Ok(_) = self.storage.disputes.get(&tx.tx) {
      return Err("Dispute already created/processed!".into());
    }
    let account = self.storage.accounts.get_mut(&tx.client)?;

    // Effects
    account.amount_available -= ref_tx_amount;
    account.amount_held += ref_tx_amount;
    self.storage.disputes.add(ref_tx.tx, Dispute::new());

    Ok(())
  }

  /// Processes resolve transaction.
  /// Validation rules:
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction has matching client ID.
  /// 4. Referenced transaction has amount associated.
  /// 5. Referenced transaction is under started dispute.
  /// 6. Client account exists.
  /// Effects:
  /// 1. Client's available amount is increased.
  /// 2. Client's held amount is decreased.
  /// 3. Dispute is marked as resolved.
  fn process_resolve(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
    // Validation
    let ref_tx = self.storage.transactions.get(&tx.tx)?;
    if ref_tx.client != tx.client {
      return Err("Mismatched client id!".into());
    }
    let ref_tx_amount = match ref_tx.amount {
      Some(x) => x,
      None => return Err("Missing amount!".into()),
    };
    let dispute = self.storage.disputes.get_mut(&tx.tx)?;
    if *dispute.get_status() != DisputeStatus::Started {
      return Err("Dispute has no 'started' state!".into());
    }
    let account = self.storage.accounts.get_mut(&tx.client)?;

    // Effects
    account.amount_available += ref_tx_amount;
    account.amount_held -= ref_tx_amount;
    dispute.mark_resolved();

    Ok(())
  }

  /// Processes chargeback transaction.
  /// Validation rules:
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction has matching client ID.
  /// 4. Referenced transaction has amount associated.
  /// 5. Referenced transaction is under started dispute.
  /// 6. Client account exists.
  /// Effects:
  /// 1. Client's held amount is decreased.
  /// 2. Client account is locked.
  /// 2. Dispute is marked as chargeback-ed.
  fn process_chargeback(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
    // Validation
    let ref_tx = self.storage.transactions.get(&tx.tx)?;
    if ref_tx.client != tx.client {
      return Err("Mismatched client id!".into());
    }
    let ref_tx_amount = match ref_tx.amount {
      Some(x) => x,
      None => return Err("Missing amount!".into()),
    };
    let dispute = self.storage.disputes.get_mut(&tx.tx)?;
    if *dispute.get_status() != DisputeStatus::Started {
      return Err("Dispute has no 'started' state!".into());
    }
    let account = self.storage.accounts.get_mut(&tx.client)?;

    // Effects
    account.amount_held -= ref_tx_amount;
    account.is_locked = true;
    dispute.mark_chargeback();

    Ok(())
  }

  /// Feeds processor with transaction and updates clients accounts accordingly.
  /// Creates new client if not already existing, but in case of any
  /// error during execution, storage will be restored to previous state.
  pub fn feed_tx(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
    // Copy client ID, before losing ownership
    let client_id = tx.client;

    // Insert client into store if not exists
    let is_new_client = !self.storage.accounts.has(&tx.client);
    if is_new_client {
      self.storage.accounts.add(Account::new(tx.client));
    }

    // Process transaction
    let process_result = match &tx.kind {
      TransactionType::Deposit => self.process_deposit(tx),
      TransactionType::Withdrawal => self.process_withdrawal(tx),
      TransactionType::Dispute => self.process_dispute(tx),
      TransactionType::Resolve => self.process_resolve(tx),
      TransactionType::Chargeback => self.process_chargeback(tx),
    };

    // Revert client insert in case of processing error
    if let Err(_) = process_result {
      if is_new_client {
        self.storage.accounts.remove(&client_id);
      }
    }

    process_result
  }

  /// Gets iterator for account map.
  pub fn get_accounts_iter(&self) -> hash_map::Iter<'_, u16, Account> {
    self.storage.accounts.raw().iter()
  }
}
