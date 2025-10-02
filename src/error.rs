use std::fmt;

#[derive(Debug)]
pub enum CliError {
    HttpError(reqwest::Error),
    ConfigurationError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::HttpError(e) => {
                write!(f, "HTTP request failed: {}", e)
            }
            CliError::ConfigurationError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
        }
    }
}

impl std::error::Error for CliError {}

impl From<reqwest::Error> for CliError {
    fn from(error: reqwest::Error) -> Self {
        CliError::HttpError(error)
    }
}
