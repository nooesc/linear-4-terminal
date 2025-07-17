pub const LINEAR_API_URL: &str = "https://api.linear.app/graphql";
pub const CONFIG_FILE: &str = ".linear-cli-config.json";

// Common GraphQL field selections
pub const ISSUE_FIELDS: &str = r#"
    id
    identifier
    title
    description
    url
    priority
    createdAt
    updatedAt
    state {
        id
        name
        type
    }
    assignee {
        id
        name
        email
    }
    team {
        id
        name
        key
    }
    labels {
        nodes {
            id
            name
            color
        }
    }
"#;

pub const PROJECT_FIELDS: &str = r#"
    id
    name
    description
    url
    createdAt
    state
    progress
"#;

pub const COMMENT_FIELDS: &str = r#"
    id
    body
    createdAt
    updatedAt
    user {
        id
        name
        email
    }
"#;