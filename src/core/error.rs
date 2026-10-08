use rust_decimal::Decimal;
use thiserror::Error;

use crate::core::types::{ClientId, TransactionId};

/// Reason why the engine refused a transaction.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EngineError {
  /// A deposit or a withdrawal carried an amount of zero or less.
  #[error("amount must be greater than 0")]
  AmountNotPositive,

  /// The available amount of the client does not fit in a **Decimal**.
  #[error("available amount of client {client} overflowed")]
  AvailableOverflowed { client: ClientId },

  /// The held amount of the client does not fit in a **Decimal**.
  #[error("held amount of client {client} overflowed")]
  HeldOverflowed { client: ClientId },

  /// The sum of the available and the held amount does not fit in a **Decimal**.
  #[error("total amount of client {client} overflowed")]
  TotalOverflowed { client: ClientId },

  /// The client has no account yet.
  #[error("account of client {client} not found")]
  AccountNotFound { client: ClientId },

  /// A chargeback locked the account, so it takes no deposit and no withdrawal.
  #[error("account of client {client} is locked")]
  AccountLocked { client: ClientId },

  /// The client does not have enough available money for the withdrawal.
  #[error("client {client} has {available} available, but the withdrawal needs {requested}")]
  NotSufficientFunds {
    client: ClientId,
    requested: Decimal,
    available: Decimal,
  },

  /// Another stored transaction already uses this ID.
  #[error("transaction {tx} is not unique")]
  TransactionNotUnique { tx: TransactionId },

  /// The referenced transaction is not stored.
  #[error("transaction {tx} not found")]
  TransactionNotFound { tx: TransactionId },

  /// The referenced transaction belongs to a different client.
  #[error("transaction {tx} belongs to client {owner}, not to client {client}")]
  MismatchedClient {
    tx: TransactionId,
    owner: ClientId,
    client: ClientId,
  },

  /// Only a deposit can be disputed, resolved or charged back.
  #[error("transaction {tx} is not a deposit")]
  NotADeposit { tx: TransactionId },

  /// The transaction is already under a dispute.
  #[error("dispute of transaction {tx} already exists")]
  DisputeAlreadyExists { tx: TransactionId },

  /// There is no dispute for the referenced transaction.
  #[error("dispute of transaction {tx} not found")]
  DisputeNotFound { tx: TransactionId },

  /// The dispute is resolved or charged back already.
  #[error("dispute of transaction {tx} is not in the started state")]
  DisputeNotStarted { tx: TransactionId },
}
