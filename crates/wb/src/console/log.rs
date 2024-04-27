use colored::Colorize;
use std::fmt::{self, Display};

#[derive(Debug, PartialEq, PartialOrd)]
pub enum Level {
    Status,
    Warning,
    Error,
    FatalError,
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Status => "status".bold().to_string(),
                Self::Warning => "warning".bold().yellow().to_string(),
                Self::Error => "error".bold().red().to_string(),
                Self::FatalError => "fatal error".bold().magenta().to_string(),
            }
        )
    }
}

pub struct Logger {
    pub min_level: Level,
}

impl Logger {
    pub fn emit(&self, level: Level, message: impl Display) {
        if level >= self.min_level {
            println!("{}: {}", level, message);
        }
    }
}
