use crate::core::error::EngineError;
use crate::core::storage::Storage;
use crate::core::types::{
  Account, ChargebackTx, DepositTx, DisputeStatus, DisputeTx, ResolveTx, StoredTx, Transaction,
  WithdrawalTx,
};

/// Balance processor, with internal storage for accounts and transactions.
/// The state of a dispute lives on the stored deposit that it refers to.
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
  fn process_deposit(&mut self, deposit: DepositTx) -> Result<(), EngineError> {
    // Validation
    if self.storage.transactions.has(deposit.tx) {
      return Err(EngineError::TransactionNotUnique { tx: deposit.tx });
    }
    // The account may not exist yet, and an unknown client is not locked.
    if let Some(account) = self.storage.accounts.get(deposit.client)
      && account.is_locked()
    {
      return Err(EngineError::AccountLocked {
        client: deposit.client,
      });
    }

    // Effects
    // A deposit is the only kind that may create an account, and it is the
    // one handler whose first effect is not the fallible one. That is safe:
    // a new account starts at zero, and a *TxAmount* is above zero and at
    // most *Decimal::MAX*, so the deposit below cannot overflow a fresh
    // account and cannot leave one behind. An existing account is not
    // created here at all.
    let account = self.storage.accounts.get_or_create(deposit.client);
    account.deposit(deposit.amount())?;
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
  fn process_withdrawal(&mut self, withdrawal: WithdrawalTx) -> Result<(), EngineError> {
    // Validation
    if self.storage.transactions.has(withdrawal.tx) {
      return Err(EngineError::TransactionNotUnique { tx: withdrawal.tx });
    }
    let account = self.storage.accounts.get_mut(withdrawal.client)?;
    if account.is_locked() {
      return Err(EngineError::AccountLocked {
        client: withdrawal.client,
      });
    }
    if account.amount_available() < withdrawal.amount().get() {
      return Err(EngineError::NotSufficientFunds {
        client: withdrawal.client,
        requested: withdrawal.amount().get(),
        available: account.amount_available(),
      });
    }

    // Effects
    account.withdraw(withdrawal.amount())?;
    self.storage.transactions.add_withdrawal(withdrawal);

    Ok(())
  }

  /// Processes dispute transaction.
  ///
  /// Validation rules:
  ///
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction is deposit.
  /// 3. Referenced transaction has matching client ID.
  /// 4. Referenced transaction is not already disputed.
  /// 5. Client account exists.
  ///
  /// Effects:
  ///
  /// 1. Client's available amount is decreased.
  /// 2. Client's held amount is increased.
  /// 3. Dispute is started on the stored deposit.
  fn process_dispute(&mut self, dispute: DisputeTx) -> Result<(), EngineError> {
    // Validation
    let StoredTx::Deposit {
      deposit: ref_deposit,
      dispute: status,
    } = self.storage.transactions.get_mut(dispute.ref_tx)?
    else {
      return Err(EngineError::NotADeposit { tx: dispute.ref_tx });
    };
    if ref_deposit.client != dispute.client {
      return Err(EngineError::MismatchedClient {
        tx: dispute.ref_tx,
        owner: ref_deposit.client,
        client: dispute.client,
      });
    }
    if status.is_some() {
      return Err(EngineError::DisputeAlreadyExists { tx: dispute.ref_tx });
    }
    let ref_tx_amount = ref_deposit.amount();
    let account = self.storage.accounts.get_mut(dispute.client)?;

    // Effects
    account.hold(ref_tx_amount)?;
    *status = Some(DisputeStatus::Started);

    Ok(())
  }

  /// Processes resolve transaction.
  ///
  /// Validation rules:
  ///
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction is deposit.
  /// 3. Referenced transaction has matching client ID.
  /// 4. Referenced transaction is under dispute.
  /// 5. That dispute is still in the started state.
  /// 6. Client account exists.
  ///
  /// Effects:
  ///
  /// 1. Client's available amount is increased.
  /// 2. Client's held amount is decreased.
  /// 3. Dispute is marked as resolved.
  fn process_resolve(&mut self, resolve: ResolveTx) -> Result<(), EngineError> {
    // Validation
    let StoredTx::Deposit {
      deposit: ref_deposit,
      dispute: status,
    } = self.storage.transactions.get_mut(resolve.ref_tx)?
    else {
      return Err(EngineError::NotADeposit { tx: resolve.ref_tx });
    };
    if ref_deposit.client != resolve.client {
      return Err(EngineError::MismatchedClient {
        tx: resolve.ref_tx,
        owner: ref_deposit.client,
        client: resolve.client,
      });
    }
    match status {
      None => return Err(EngineError::DisputeNotFound { tx: resolve.ref_tx }),
      Some(DisputeStatus::Started) => {}
      Some(_) => return Err(EngineError::DisputeNotStarted { tx: resolve.ref_tx }),
    }
    let ref_tx_amount = ref_deposit.amount();
    let account = self.storage.accounts.get_mut(resolve.client)?;

    // Effects
    account.release(ref_tx_amount)?;
    *status = Some(DisputeStatus::Resolved);

    Ok(())
  }

  /// Processes chargeback transaction.
  ///
  /// Validation rules:
  ///
  /// 1. Referenced transaction exists.
  /// 2. Referenced transaction is deposit.
  /// 3. Referenced transaction has matching client ID.
  /// 4. Referenced transaction is under dispute.
  /// 5. That dispute is still in the started state.
  /// 6. Client account exists.
  ///
  /// Effects:
  ///
  /// 1. Client's held amount is decreased.
  /// 2. Client account is locked.
  /// 3. Dispute is marked as charged back.
  fn process_chargeback(&mut self, chargeback: ChargebackTx) -> Result<(), EngineError> {
    // Validation
    let StoredTx::Deposit {
      deposit: ref_deposit,
      dispute: status,
    } = self.storage.transactions.get_mut(chargeback.ref_tx)?
    else {
      return Err(EngineError::NotADeposit {
        tx: chargeback.ref_tx,
      });
    };
    if ref_deposit.client != chargeback.client {
      return Err(EngineError::MismatchedClient {
        tx: chargeback.ref_tx,
        owner: ref_deposit.client,
        client: chargeback.client,
      });
    }
    match status {
      None => {
        return Err(EngineError::DisputeNotFound {
          tx: chargeback.ref_tx,
        });
      }
      Some(DisputeStatus::Started) => {}
      Some(_) => {
        return Err(EngineError::DisputeNotStarted {
          tx: chargeback.ref_tx,
        });
      }
    }
    let ref_tx_amount = ref_deposit.amount();
    let account = self.storage.accounts.get_mut(chargeback.client)?;

    // Effects
    account.withdraw_held(ref_tx_amount)?;
    account.lock();
    *status = Some(DisputeStatus::ChargedBack);

    Ok(())
  }

  /// Feeds processor with transaction and updates clients accounts accordingly.
  pub fn feed_tx(&mut self, tx: Transaction) -> Result<(), EngineError> {
    match tx {
      Transaction::Deposit(details) => self.process_deposit(details),
      Transaction::Withdrawal(details) => self.process_withdrawal(details),
      Transaction::Dispute(details) => self.process_dispute(details),
      Transaction::Resolve(details) => self.process_resolve(details),
      Transaction::Chargeback(details) => self.process_chargeback(details),
    }
  }

  /// Gets iterator over the stored accounts.
  pub fn accounts(&self) -> impl Iterator<Item = &Account> {
    self.storage.accounts.iter()
  }
}
