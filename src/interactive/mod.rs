pub mod app;
pub mod ui;
pub mod event;
pub mod handlers;
pub mod state;
pub mod state_adapter;

// Example usage of the new state system (compile with --features examples)
#[cfg(feature = "examples")]
pub mod state_example;