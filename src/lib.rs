/// Module containing engine for processing transactions
/// and calculating user balances.
///
pub mod core;

/// Module that is wrapper over **core** that adds possibility
/// to read transactions from *.csv* files and print account
/// summary in *.csv* format as well.
pub mod csv;
