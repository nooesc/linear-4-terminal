#![allow(dead_code)]

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LinearError {
    #[error("API key not found. Please run 'linear auth' to configure.")]
    ApiKeyNotFound,
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("API request failed: {0}")]
    ApiError(String),
    
    #[error("GraphQL error: {0}")]
    GraphQLError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Terminal error: {0}")]
    TerminalError(String),
    
    #[error("State error: {0}")]
    StateError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type LinearResult<T> = Result<T, LinearError>;

pub trait ErrorContext<T> {
    fn context(self, msg: &str) -> LinearResult<T>;
    fn with_context<F>(self, f: F) -> LinearResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: std::error::Error + 'static,
{
    fn context(self, msg: &str) -> LinearResult<T> {
        self.map_err(|e| LinearError::Unknown(format!("{}: {}", msg, e)))
    }
    
    fn with_context<F>(self, f: F) -> LinearResult<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| LinearError::Unknown(format!("{}: {}", f(), e)))
    }
}

impl<T> ErrorContext<T> for Option<T> {
    fn context(self, msg: &str) -> LinearResult<T> {
        self.ok_or_else(|| LinearError::Unknown(msg.to_string()))
    }
    
    fn with_context<F>(self, f: F) -> LinearResult<T>
    where
        F: FnOnce() -> String,
    {
        self.ok_or_else(|| LinearError::Unknown(f()))
    }
}

#[macro_export]
macro_rules! linear_error {
    ($error_type:ident, $msg:expr) => {
        LinearError::$error_type($msg.to_string())
    };
    ($error_type:ident, $fmt:expr, $($arg:tt)*) => {
        LinearError::$error_type(format!($fmt, $($arg)*))
    };
}