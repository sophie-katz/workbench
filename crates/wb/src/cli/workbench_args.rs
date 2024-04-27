use std::collections::HashSet;

use lazy_static::lazy_static;

use clap::Parser;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct WorkbenchArgs {
    /// The number of tasks to run in parallel
    #[arg(short, long, default_value_t = 0)]
    pub jobs: u32,

    /// Whether or not to enable verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

lazy_static! {
    pub static ref ARGS_WITH_VALUES: HashSet<String> =
        HashSet::from(["-j".to_owned(), "--jobs".to_owned(),]);
}
