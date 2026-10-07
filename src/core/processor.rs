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
  ///
  /// Validation rules:
  ///
  /// 1. Transaction ID is unique.
  /// 2. Client account exists.
  /// 3. Account is not locked.
  ///
  /// Effects:
  ///
  /// 1. Client's available amount is increased.
  /// 2. Transaction is stored.
  fn process_deposit(&mut self, deposit: DepositTx) -> Result<(), Box<dyn Error>> {
    // Validation
    self.storage.transactions.assert_unique_id(&deposit.tx)?;
    let account = self.storage.accounts.get_mut(&deposit.client)?;
    if account.is_locked {
      return Err("Account is locked!".into());
    }

    // Effects
    account.deposit(deposit.amount)?;
    self.storage.transactions.add_deposit(deposit);

    Ok(())
  }

  /// Processes withdrawal transaction.
  ///
  /// Validation rules:
  ///
  /// 1. Transaction ID is unique.
  /// 2. Client account exists.
  /// 3. Account is not locked.
  /// 4. Client has enough money available.
  ///
  /// Effects:
  ///
  /// 1. Client's available amount is decreased.
  /// 2. Transaction is stored.
  fn process_withdrawal(&mut self, withdrawal: WithdrawalTx) -> Result<(), Box<dyn Error>> {
    // Validation
    self.storage.transactions.assert_unique_id(&withdrawal.tx)?;
    let account = self.storage.accounts.get_mut(&withdrawal.client)?;
    if account.is_locked {
      return Err("Account is locked!".into());
    }
    if account.amount_available < withdrawal.amount {
      return Err("Not sufficient funds to make withdrawal".into());
    }

    // Effects
    account.withdraw(withdrawal.amount)?;
    self.storage.transactions.add_withdrawal(withdrawal);

    Ok(())
  }

  /// Processes dispute transaction.
  ///
  /// Validation rules:
  ///
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction has matching client ID.
  /// 3. Referenced transaction is deposit.
  /// 4. Referenced transaction has amount associated.
  /// 5. Referenced transaction is not already disputed.
  /// 6. Client account exists.
  ///
  /// Effects:
  ///
  /// 1. Client's available amount is decreased.
  /// 2. Client's held amount is increased.
  /// 3. Dispute is stored.
  fn process_dispute(&mut self, dispute: DisputeTx) -> Result<(), Box<dyn Error>> {
    // Validation
    let ref_tx = self.storage.transactions.get(&dispute.ref_tx)?;
    if ref_tx.client() != dispute.client {
      return Err("Mismatched client id!".into());
    }
    // A dispute refers to a deposit, and takes the deposited amount
    let Transaction::Deposit(ref_deposit) = ref_tx else {
      return Err("Cannot dispute transaction different than deposit".into());
    };
    let ref_tx_amount = ref_deposit.amount;
    if self.storage.disputes.get(&dispute.ref_tx).is_ok() {
      return Err("Dispute already created/processed!".into());
    }
    let account = self.storage.accounts.get_mut(&dispute.client)?;

    // Effects
    account.hold(ref_tx_amount)?;
    self
      .storage
      .disputes
      .add(dispute.ref_tx, DisputeStatus::Started);

    Ok(())
  }

  /// Processes resolve transaction.
  ///
  /// Validation rules:
  ///
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction has matching client ID.
  /// 3. Referenced transaction has amount associated.
  /// 4. Referenced transaction is under started dispute.
  /// 5. Client account exists.
  ///
  /// Effects:
  ///
  /// 1. Client's available amount is increased.
  /// 2. Client's held amount is decreased.
  /// 3. Dispute is marked as resolved.
  fn process_resolve(&mut self, resolve: ResolveTx) -> Result<(), Box<dyn Error>> {
    // Validation
    let ref_tx = self.storage.transactions.get(&resolve.ref_tx)?;
    if ref_tx.client() != resolve.client {
      return Err("Mismatched client id!".into());
    }
    // A resolve refers to a disputed deposit, and takes the deposited amount
    let Transaction::Deposit(ref_deposit) = ref_tx else {
      return Err("Cannot resolve transaction different than deposit".into());
    };
    let ref_tx_amount = ref_deposit.amount;
    let dispute = self.storage.disputes.get_mut(&resolve.ref_tx)?;
    if *dispute != DisputeStatus::Started {
      return Err("Dispute has no 'started' state!".into());
    }
    let account = self.storage.accounts.get_mut(&resolve.client)?;

    // Effects
    account.release(ref_tx_amount)?;
    *dispute = DisputeStatus::Resolved;

    Ok(())
  }

  /// Processes chargeback transaction.
  ///
  /// Validation rules:
  ///
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction has matching client ID.
  /// 3. Referenced transaction has amount associated.
  /// 4. Referenced transaction is under started dispute.
  /// 5. Client account exists.
  ///
  /// Effects:
  ///
  /// 1. Client's held amount is decreased.
  /// 2. Client account is locked.
  /// 3. Dispute is marked as chargeback-ed.
  fn process_chargeback(&mut self, chargeback: ChargebackTx) -> Result<(), Box<dyn Error>> {
    // Validation
    let ref_tx = self.storage.transactions.get(&chargeback.ref_tx)?;
    if ref_tx.client() != chargeback.client {
      return Err("Mismatched client id!".into());
    }
    // A chargeback refers to a disputed deposit, and takes the deposited amount
    let Transaction::Deposit(ref_deposit) = ref_tx else {
      return Err("Cannot resolve transaction different than deposit".into());
    };
    let ref_tx_amount = ref_deposit.amount;
    let dispute = self.storage.disputes.get_mut(&chargeback.ref_tx)?;
    if *dispute != DisputeStatus::Started {
      return Err("Dispute has no 'started' state!".into());
    }
    let account = self.storage.accounts.get_mut(&chargeback.client)?;

    // Effects
    account.withdraw_held(ref_tx_amount)?;
    account.is_locked = true;
    *dispute = DisputeStatus::Chargeback;

    Ok(())
  }

  /// Feeds processor with transaction and updates clients accounts accordingly.
  /// Creates new client if not already existing, but in case of any
  /// error during execution, storage will be restored to previous state.
  pub fn feed_tx(&mut self, tx: Transaction) -> Result<(), Box<dyn Error>> {
    // Copy client ID, before losing ownership
    let client_id = tx.client();

    // Insert client into store if not exists
    let is_new_client = !self.storage.accounts.has(&client_id);
    if is_new_client {
      self.storage.accounts.add(Account::new(client_id));
    }

    // Process transaction
    let process_result = match tx {
      Transaction::Deposit(details) => self.process_deposit(details),
      Transaction::Withdrawal(details) => self.process_withdrawal(details),
      Transaction::Dispute(details) => self.process_dispute(details),
      Transaction::Resolve(details) => self.process_resolve(details),
      Transaction::Chargeback(details) => self.process_chargeback(details),
    };

    // Revert client insert in case of processing error
    if process_result.is_err() && is_new_client {
      self.storage.accounts.remove(&client_id);
    }

    process_result
  }

  /// Gets iterator for account map.
  pub fn get_accounts_iter(&self) -> impl Iterator<Item = (&ClientId, &Account)> {
    self.storage.accounts.raw().iter()
  }
}
