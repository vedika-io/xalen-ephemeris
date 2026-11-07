//! Error type and process exit codes.

use std::fmt;

use xalen_ephemeris::ephem::EphemerisError;

/// A failed command, split by the exit code it maps to.
#[derive(Debug)]
pub enum CliError {
    /// The input was wrong: a bad date, an unknown body, a missing option.
    /// Exit code 2.
    Usage(String),
    /// The input was fine but the library could not compute the answer.
    /// Exit code 1.
    Compute(String),
}

impl CliError {
    /// Build a [`CliError::Usage`].
    pub fn usage(message: impl Into<String>) -> Self {
        CliError::Usage(message.into())
    }

    /// Build a [`CliError::Compute`].
    pub fn compute(message: impl Into<String>) -> Self {
        CliError::Compute(message.into())
    }

    /// The process exit code for this error.
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Usage(_) => 2,
            CliError::Compute(_) => 1,
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            CliError::Usage(m) | CliError::Compute(m) => m,
        };
        // One line on stderr, always.
        write!(f, "{}", message.replace('\n', " "))
    }
}

impl std::error::Error for CliError {}

impl From<EphemerisError> for CliError {
    fn from(error: EphemerisError) -> Self {
        CliError::Compute(error.to_string())
    }
}

/// Result alias used by every command.
pub type Result<T> = std::result::Result<T, CliError>;
