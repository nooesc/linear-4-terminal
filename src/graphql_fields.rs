use std::collections::HashSet;
use std::fmt;

/// Type-safe GraphQL field selection builder
#[derive(Debug, Clone)]
pub struct FieldSelection {
    fields: HashSet<String>,
}

impl FieldSelection {
    pub fn new() -> Self {
        Self {
            fields: HashSet::new(),
        }
    }
    
    /// Add a simple field
    pub fn field(mut self, name: &str) -> Self {
        self.fields.insert(name.to_string());
        self
    }
    
    /// Add multiple simple fields
    pub fn fields(mut self, names: &[&str]) -> Self {
        for name in names {
            self.fields.insert(name.to_string());
        }
        self
    }
    
    /// Add a nested field with its own selection
    pub fn nested(mut self, name: &str, selection: FieldSelection) -> Self {
        let nested_str = format!("{} {{ {} }}", name, selection);
        self.fields.insert(nested_str);
        self
    }
    
    /// Add a field with arguments
    pub fn field_with_args(mut self, name: &str, args: &[(&str, &str)]) -> Self {
        let args_str = args
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        let field_str = format!("{}({})", name, args_str);
        self.fields.insert(field_str);
        self
    }
    
    /// Add a nested field with arguments
    pub fn nested_with_args(
        mut self,
        name: &str,
        args: &[(&str, &str)],
        selection: FieldSelection,
    ) -> Self {
        let args_str = args
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        let field_str = format!("{}({}) {{ {} }}", name, args_str, selection);
        self.fields.insert(field_str);
        self
    }
    
    /// Merge another field selection into this one
    pub fn merge(mut self, other: FieldSelection) -> Self {
        self.fields.extend(other.fields);
        self
    }
}

impl fmt::Display for FieldSelection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fields_str = self.fields.iter().cloned().collect::<Vec<_>>().join(" ");
        write!(f, "{}", fields_str)
    }
}

impl Default for FieldSelection {
    fn default() -> Self {
        Self::new()
    }
}

/// Predefined field selections for common entities
pub mod selections {
    use super::FieldSelection;
    
    pub fn user_fields() -> FieldSelection {
        FieldSelection::new()
            .fields(&["id", "name", "email", "displayName", "avatarUrl"])
    }
    
    pub fn workflow_state_fields() -> FieldSelection {
        FieldSelection::new()
            .fields(&["id", "name", "type", "color", "position", "description"])
    }
    
    pub fn label_fields() -> FieldSelection {
        FieldSelection::new()
            .fields(&["id", "name", "color", "description"])
            .nested("parent", FieldSelection::new().fields(&["id", "name"]))
    }
    
    pub fn project_fields() -> FieldSelection {
        FieldSelection::new()
            .fields(&[
                "id", "name", "description", "icon", "color", "state",
                "targetDate", "startedAt", "completedAt", "canceledAt",
                "createdAt", "updatedAt", "progress", "url"
            ])
            .nested("lead", user_fields())
    }
    
    pub fn issue_fields() -> FieldSelection {
        FieldSelection::new()
            .fields(&[
                "id", "identifier", "title", "description", "priority",
                "createdAt", "updatedAt", "completedAt", "canceledAt",
                "url", "branchName", "estimate"
            ])
            .nested("assignee", user_fields())
            .nested("creator", user_fields())
            .nested("state", workflow_state_fields())
            .nested("project", FieldSelection::new().fields(&["id", "name", "icon", "color"]))
            .nested("parent", FieldSelection::new().fields(&["id", "identifier", "title"]))
            .nested_with_args("labels", &[("first", "50")], 
                FieldSelection::new()
                    .nested("nodes", label_fields())
            )
            .nested_with_args("comments", &[("first", "10")],
                FieldSelection::new()
                    .nested("nodes", comment_fields())
            )
    }
    
    pub fn comment_fields() -> FieldSelection {
        FieldSelection::new()
            .fields(&["id", "body", "createdAt", "updatedAt", "url"])
            .nested("user", user_fields())
    }
    
    pub fn team_fields() -> FieldSelection {
        FieldSelection::new()
            .fields(&["id", "name", "key", "description", "icon", "color"])
    }
}

/// Macro for building GraphQL queries with type safety
#[macro_export]
macro_rules! graphql_query {
    ($query_name:expr, $selection:expr) => {
        format!("query {{ {} {{ {} }} }}", $query_name, $selection)
    };
    ($query_name:expr, $args:expr, $selection:expr) => {
        format!("query {{ {}({}) {{ {} }} }}", $query_name, $args, $selection)
    };
}

/// Macro for building GraphQL mutations with type safety
#[macro_export]
macro_rules! graphql_mutation {
    ($mutation_name:expr, $input_type:expr, $input:expr, $selection:expr) => {
        format!(
            "mutation {{ {}(input: {}) {{ {} }} }}",
            $mutation_name,
            serde_json::to_string(&$input).unwrap(),
            $selection
        )
    };
    ($mutation_name:expr, $args:expr, $selection:expr) => {
        format!("mutation {{ {}({}) {{ {} }} }}", $mutation_name, $args, $selection)
    };
}