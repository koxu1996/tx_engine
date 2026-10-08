mod cli;

use std::io;
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
  let report = provider.load_from_path(config.input)?;
  if report.skipped > 0 {
    eprintln!(
      "Skipped {} of {} rows.",
      report.skipped,
      report.accepted + report.skipped
    );
  }

  // Print summary of each client - balance, lock status.
  provider.write_accounts_summary(csv::SummaryOrder::ByClientId, io::stdout())?;

  Ok(())
}
