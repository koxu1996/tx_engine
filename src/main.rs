mod cli;

use std::process;

use tx_engine::csv;

fn main() {
  // Load program configuration from args.
  let config = cli::parse_args();

  // Load given CSV into transaction processor.
  let mut provider = csv::CsvProvider::default();
  provider.load_from_path(config.input).unwrap_or_else(|err| {
    eprintln!("Unable to load CSV file: {}", err);
    process::exit(1);
  });

  // Print summary of each client - balance, lock status.
  provider.print_accounts_summary().unwrap_or_else(|err| {
    eprintln!("Unable to print accounts summary: {}", err);
    process::exit(1);
  });
}
