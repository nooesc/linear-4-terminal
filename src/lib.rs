// Module declarations
pub mod client;
pub mod commands;
pub mod config;
pub mod constants;
pub mod filtering;
pub mod formatting;
pub mod models;
pub mod error;
pub mod cli_context;
pub mod graphql_fields;

#[cfg(test)]
mod tests;

// Re-export commonly used items
pub use client::LinearClient;
pub use config::{Config, get_api_key, load_config, save_config};
pub use models::*;
pub use error::{LinearError, LinearResult};
pub use cli_context::{CliContext, CliContextBuilder};