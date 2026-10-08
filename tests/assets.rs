use std::fs;

use tx_engine::csv::{CsvProvider, SummaryOrder};

/// Runs every *.csv* file of the _./assets_ directory through the engine,
/// and compares the summary with the *.expected* file next to it.
///
/// The summary asks for the *ByClientId* order, so the comparison does not
/// depend on the iteration order of a map.
#[test]
fn every_asset_matches_its_expected_summary() {
  let mut checked = 0;

  for entry in fs::read_dir("assets").expect("the assets directory is missing") {
    let input = entry.expect("cannot read the assets directory").path();
    if input.extension().and_then(|extension| extension.to_str()) != Some("csv") {
      continue;
    }

    let expected_path = input.with_extension("expected");
    let expected = fs::read_to_string(&expected_path)
      .unwrap_or_else(|_| panic!("{} has no expected file", input.display()));

    let mut provider = CsvProvider::default();
    provider
      .load_from_path(&input)
      .unwrap_or_else(|error| panic!("{}: {error}", input.display()));

    let mut summary = Vec::new();
    provider
      .write_accounts_summary(SummaryOrder::ByClientId, &mut summary)
      .unwrap_or_else(|error| panic!("{}: {error}", input.display()));

    let summary = String::from_utf8(summary).expect("the summary is not valid UTF-8");
    assert_eq!(
      summary,
      expected,
      "unexpected summary for {}",
      input.display()
    );

    checked += 1;
  }

  assert!(
    checked > 0,
    "no .csv file was found in the assets directory"
  );
}
