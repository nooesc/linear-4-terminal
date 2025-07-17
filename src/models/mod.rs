pub mod comment;
pub mod graphql;
pub mod issue;
pub mod project;
pub mod user;

// Re-export commonly used types
pub use comment::Comment;
pub use graphql::GraphQLResponse;
pub use issue::{Issue, WorkflowState};
pub use project::Project;
pub use user::{Team, User};

// Connection type used by GraphQL pagination
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Connection<T> {
    pub nodes: Vec<T>,
}