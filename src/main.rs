mod cli;

use std::process::ExitCode;

use tx_engine::csv;
use tx_engine::csv::error::CsvError;

fn main() -> ExitCode {
  if let Err(err) = run() {
    eprintln!("{err}");
    return ExitCode::FAILURE;
  }

  ExitCode::SUCCESS
}

/// Runs the engine.
fn run() -> Result<(), CsvError> {
  // Load program configuration from args.
  let config = cli::parse_args();

  // Load given CSV into transaction processor.
  let mut provider = csv::CsvProvider::default();
  provider.load_from_path(config.input)?;

  // Print summary of each client - balance, lock status.
  provider.print_accounts_summary(csv::SummaryOrder::ByClientId)?;

  Ok(())
}
