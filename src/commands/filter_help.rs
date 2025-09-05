use clap::ArgMatches;
use crate::filtering::print_filter_examples;

/// Handle the filter-help command
pub async fn handle_filter_help(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    print_filter_examples();
    Ok(())
}