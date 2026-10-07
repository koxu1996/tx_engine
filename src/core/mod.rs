/// Module with the error type of the engine.
pub mod error;

/// Module containing logic for processing transactions
/// and calculating user balances.
pub mod processor;

/// Module with base types.
pub mod types;

/// Internal module, used to define account/transaction/dispute storage.
mod storage;
