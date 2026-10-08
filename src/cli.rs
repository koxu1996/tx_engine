use std::path::PathBuf;

use clap::Parser;

/// Application for processing transactions.
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct AppConfig {
  /// Path to .csv file containing transactions.
  #[arg(value_name = "TX_FILE")]
  pub input: PathBuf,
}

/// Constructs *AppConfig* by parsing program args, exits on error.
pub fn parse_args() -> AppConfig {
  AppConfig::parse()
}
