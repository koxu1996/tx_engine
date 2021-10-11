use std::path::PathBuf;

use structopt::{clap, StructOpt};

/// Application for processing transactions.
#[derive(StructOpt, Debug)]
#[structopt()]
pub struct AppConfig {
  /// Path to .csv file containing transactions.
  #[structopt(name = "TX_FILE", parse(from_os_str))]
  pub input: PathBuf,
}

/// Constructs *AppConfig* by parsing program args.
pub fn parse_args() -> Result<AppConfig, clap::Error> {
  AppConfig::from_args_safe()
}
