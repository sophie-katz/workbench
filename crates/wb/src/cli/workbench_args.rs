use std::collections::HashSet;

use lazy_static::lazy_static;

use clap::Parser;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
pub struct WorkbenchArgs {
    /// Name of the person to greet
    #[arg(short, long)]
    pub name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    pub count: u8,
}

lazy_static! {
    pub static ref ARGS_WITH_VALUES: HashSet<String> = HashSet::from([
        "-n".to_owned(),
        "--name".to_owned(),
        "-c".to_owned(),
        "--count".to_owned()
    ]);
}
