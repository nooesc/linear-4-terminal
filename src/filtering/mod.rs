pub mod query;
pub mod builder;
pub mod parser;
pub mod adapter;

// Legacy exports for backward compatibility
pub use query::{parse_filter_query, build_graphql_filter};

// New exports
pub use builder::{FilterBuilder, FilterField, FilterOperator, FilterValue, FilterError};
pub use parser::{parse_filter, ParseError};
pub use adapter::{FilterAdapter, print_filter_examples};