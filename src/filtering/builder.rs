#![allow(dead_code)]

use chrono::{Duration, Utc};
use serde_json::{json, Value};

/// Represents a single filter condition
#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub field: FilterField,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

/// Supported filter fields
#[derive(Debug, Clone, PartialEq)]
pub enum FilterField {
    Title,
    Description,
    Status,
    Priority,
    Assignee,
    Label,
    Project,
    Team,
    CreatedAt,
    UpdatedAt,
    DueDate,
    Identifier,
    Custom(String),
}

impl FilterField {
    /// Get the GraphQL field name
    pub fn field_name(&self) -> &str {
        match self {
            Self::Title => "title",
            Self::Description => "description",
            Self::Status => "state",
            Self::Priority => "priority",
            Self::Assignee => "assignee",
            Self::Label => "labels",
            Self::Project => "project",
            Self::Team => "team",
            Self::CreatedAt => "createdAt",
            Self::UpdatedAt => "updatedAt",
            Self::DueDate => "dueDate",
            Self::Identifier => "identifier",
            Self::Custom(name) => name,
        }
    }
}

/// Filter operators
#[derive(Debug, Clone, PartialEq)]
pub enum FilterOperator {
    // Equality
    Equals,
    NotEquals,
    
    // Comparison
    GreaterThan,
    GreaterThanOrEquals,
    LessThan,
    LessThanOrEquals,
    
    // String matching
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    
    // Collection operators
    In,
    NotIn,
    
    // Existence
    IsNull,
    IsNotNull,
    
    // Special operators
    HasAny,
    HasAll,
    HasNone,
}

/// Filter value types
#[derive(Debug, Clone)]
pub enum FilterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Date(String),
    RelativeDate(Duration),
    StringList(Vec<String>),
    NumberList(Vec<f64>),
    Null,
}

/// Logical operators for combining filters
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// A group of filters combined with a logical operator
#[derive(Debug, Clone)]
pub struct FilterGroup {
    pub operator: LogicalOperator,
    pub conditions: Vec<FilterExpression>,
}

/// Filter expression can be a single condition or a group
#[derive(Debug, Clone)]
pub enum FilterExpression {
    Condition(FilterCondition),
    Group(Box<FilterGroup>),
}

/// Builder for creating complex filter expressions
pub struct FilterBuilder {
    root: Option<FilterExpression>,
    current_group: Vec<FilterExpression>,
    current_operator: LogicalOperator,
}

impl FilterBuilder {
    /// Create a new filter builder
    pub fn new() -> Self {
        Self {
            root: None,
            current_group: Vec::new(),
            current_operator: LogicalOperator::And,
        }
    }

    /// Create a filter builder from a pre-built expression tree.
    pub(crate) fn from_expression(root: FilterExpression) -> Self {
        Self {
            root: Some(root),
            current_group: Vec::new(),
            current_operator: LogicalOperator::And,
        }
    }
    
    /// Add a condition to the current group
    fn add_condition(&mut self, condition: FilterCondition) -> &mut Self {
        self.current_group.push(FilterExpression::Condition(condition));
        self
    }
    
    /// Start building a field filter
    pub fn field(&mut self, field: FilterField) -> FieldBuilder<'_> {
        FieldBuilder {
            builder: self,
            field,
        }
    }
    
    // Convenience methods for common fields
    pub fn title(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::Title)
    }
    
    pub fn description(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::Description)
    }
    
    pub fn status(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::Status)
    }
    
    pub fn priority(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::Priority)
    }
    
    pub fn assignee(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::Assignee)
    }
    
    pub fn label(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::Label)
    }
    
    pub fn project(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::Project)
    }
    
    pub fn created_at(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::CreatedAt)
    }
    
    pub fn updated_at(&mut self) -> FieldBuilder<'_> {
        self.field(FilterField::UpdatedAt)
    }
    
    /// Combine filters with AND
    pub fn and(&mut self) -> &mut Self {
        self.current_operator = LogicalOperator::And;
        self
    }
    
    /// Combine filters with OR
    pub fn or(&mut self) -> &mut Self {
        self.current_operator = LogicalOperator::Or;
        self
    }
    
    /// Start a new group with AND
    pub fn and_group(&mut self) -> &mut Self {
        self.start_group(LogicalOperator::And)
    }
    
    /// Start a new group with OR
    pub fn or_group(&mut self) -> &mut Self {
        self.start_group(LogicalOperator::Or)
    }
    
    /// Start a new group with NOT
    pub fn not_group(&mut self) -> &mut Self {
        self.start_group(LogicalOperator::Not)
    }
    
    /// Start a new group
    fn start_group(&mut self, operator: LogicalOperator) -> &mut Self {
        // Save current group if any
        if !self.current_group.is_empty() {
            let group = FilterGroup {
                operator: self.current_operator.clone(),
                conditions: std::mem::take(&mut self.current_group),
            };
            
            if self.root.is_none() {
                self.root = Some(FilterExpression::Group(Box::new(group)));
            } else {
                // This would need more complex handling for nested groups
                self.current_group = vec![self.root.take().unwrap(), FilterExpression::Group(Box::new(group))];
                self.root = None;
            }
        }
        
        self.current_operator = operator;
        self
    }
    
    /// End the current group
    pub fn end_group(&mut self) -> &mut Self {
        self
    }
    
    /// Build the final filter expression
    pub fn build(self) -> Result<FilterExpression, FilterError> {
        if self.current_group.is_empty() && self.root.is_none() {
            return Err(FilterError::EmptyFilter);
        }
        
        if !self.current_group.is_empty() {
            let group = FilterGroup {
                operator: self.current_operator,
                conditions: self.current_group,
            };
            
            if let Some(root) = self.root {
                // Combine root and current group
                Ok(FilterExpression::Group(Box::new(FilterGroup {
                    operator: LogicalOperator::And,
                    conditions: vec![root, FilterExpression::Group(Box::new(group))],
                })))
            } else {
                Ok(FilterExpression::Group(Box::new(group)))
            }
        } else {
            Ok(self.root.unwrap())
        }
    }
    
    /// Convert to GraphQL filter format
    pub fn to_graphql(self) -> Result<Value, FilterError> {
        let expr = self.build()?;
        Ok(expression_to_graphql(&expr))
    }
}

/// Builder for field-specific operations
pub struct FieldBuilder<'a> {
    builder: &'a mut FilterBuilder,
    field: FilterField,
}

impl<'a> FieldBuilder<'a> {
    // Equality operators
    pub fn equals(self, value: impl Into<FilterValue>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::Equals,
            value: value.into(),
        })
    }
    
    pub fn not_equals(self, value: impl Into<FilterValue>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::NotEquals,
            value: value.into(),
        })
    }
    
    // Comparison operators
    pub fn greater_than(self, value: impl Into<FilterValue>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::GreaterThan,
            value: value.into(),
        })
    }
    
    pub fn greater_than_or_equals(self, value: impl Into<FilterValue>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::GreaterThanOrEquals,
            value: value.into(),
        })
    }
    
    pub fn less_than(self, value: impl Into<FilterValue>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::LessThan,
            value: value.into(),
        })
    }
    
    pub fn less_than_or_equals(self, value: impl Into<FilterValue>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::LessThanOrEquals,
            value: value.into(),
        })
    }
    
    // String operators
    pub fn contains(self, value: impl Into<String>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::Contains,
            value: FilterValue::String(value.into()),
        })
    }
    
    pub fn not_contains(self, value: impl Into<String>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::NotContains,
            value: FilterValue::String(value.into()),
        })
    }
    
    pub fn starts_with(self, value: impl Into<String>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::StartsWith,
            value: FilterValue::String(value.into()),
        })
    }
    
    pub fn ends_with(self, value: impl Into<String>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::EndsWith,
            value: FilterValue::String(value.into()),
        })
    }
    
    // Collection operators
    pub fn in_list(self, values: Vec<String>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::In,
            value: FilterValue::StringList(values),
        })
    }
    
    pub fn not_in_list(self, values: Vec<String>) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::NotIn,
            value: FilterValue::StringList(values),
        })
    }
    
    // Existence operators
    pub fn is_null(self) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::IsNull,
            value: FilterValue::Null,
        })
    }
    
    pub fn is_not_null(self) -> &'a mut FilterBuilder {
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::IsNotNull,
            value: FilterValue::Null,
        })
    }
    
    // Date helpers
    pub fn within_days(self, days: i64) -> &'a mut FilterBuilder {
        let duration = Duration::days(days);
        let date = Utc::now() - duration;
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::GreaterThanOrEquals,
            value: FilterValue::Date(date.to_rfc3339()),
        })
    }
    
    pub fn older_than_days(self, days: i64) -> &'a mut FilterBuilder {
        let duration = Duration::days(days);
        let date = Utc::now() - duration;
        self.builder.add_condition(FilterCondition {
            field: self.field,
            operator: FilterOperator::LessThan,
            value: FilterValue::Date(date.to_rfc3339()),
        })
    }
}

/// Convert expression to GraphQL filter
fn expression_to_graphql(expr: &FilterExpression) -> Value {
    match expr {
        FilterExpression::Condition(condition) => condition_to_graphql(condition),
        FilterExpression::Group(group) => group_to_graphql(group),
    }
}

/// Convert a filter group to GraphQL
fn group_to_graphql(group: &FilterGroup) -> Value {
    match &group.operator {
        LogicalOperator::And => {
            let mut combined = json!({});
            for expr in &group.conditions {
                let value = expression_to_graphql(expr);
                if let Some(obj) = value.as_object() {
                    for (k, v) in obj {
                        combined[k] = v.clone();
                    }
                }
            }
            combined
        }
        LogicalOperator::Or => {
            json!({
                "or": group.conditions.iter()
                    .map(expression_to_graphql)
                    .collect::<Vec<_>>()
            })
        }
        LogicalOperator::Not => {
            json!({
                "not": group.conditions.iter()
                    .map(expression_to_graphql)
                    .collect::<Vec<_>>()
            })
        }
    }
}

/// Convert a single condition to GraphQL
fn condition_to_graphql(condition: &FilterCondition) -> Value {
    let field_name = condition.field.field_name();
    
    match (&condition.field, &condition.operator, &condition.value) {
        // Title operations
        (FilterField::Title, FilterOperator::Contains, FilterValue::String(s)) => {
            json!({ field_name: { "containsIgnoreCase": s } })
        }
        (FilterField::Title, FilterOperator::NotContains, FilterValue::String(s)) => {
            json!({ field_name: { "not": { "containsIgnoreCase": s } } })
        }
        (FilterField::Title, FilterOperator::StartsWith, FilterValue::String(s)) => {
            json!({ field_name: { "startsWithIgnoreCase": s } })
        }
        
        // Status operations
        (FilterField::Status, FilterOperator::Equals, FilterValue::String(s)) => {
            json!({ field_name: { "name": { "eq": s } } })
        }
        (FilterField::Status, FilterOperator::NotEquals, FilterValue::String(s)) => {
            json!({ field_name: { "name": { "neq": s } } })
        }
        (FilterField::Status, FilterOperator::In, FilterValue::StringList(list)) => {
            json!({ field_name: { "name": { "in": list } } })
        }
        
        // Priority operations
        (FilterField::Priority, FilterOperator::Equals, FilterValue::Number(n)) => {
            json!({ field_name: { "eq": n } })
        }
        (FilterField::Priority, FilterOperator::NotEquals, FilterValue::Number(n)) => {
            json!({ field_name: { "neq": n } })
        }
        (FilterField::Priority, FilterOperator::GreaterThan, FilterValue::Number(n)) => {
            json!({ field_name: { "gt": n } })
        }
        (FilterField::Priority, FilterOperator::GreaterThanOrEquals, FilterValue::Number(n)) => {
            json!({ field_name: { "gte": n } })
        }
        (FilterField::Priority, FilterOperator::LessThan, FilterValue::Number(n)) => {
            json!({ field_name: { "lt": n } })
        }
        (FilterField::Priority, FilterOperator::LessThanOrEquals, FilterValue::Number(n)) => {
            json!({ field_name: { "lte": n } })
        }
        
        // Assignee operations
        (FilterField::Assignee, FilterOperator::Equals, FilterValue::String(s)) => {
            json!({ field_name: { "email": { "eq": s } } })
        }
        (FilterField::Assignee, FilterOperator::IsNull, _) => {
            json!({ field_name: { "null": true } })
        }
        (FilterField::Assignee, FilterOperator::IsNotNull, _) => {
            json!({ field_name: { "null": false } })
        }
        
        // Label operations
        (FilterField::Label, FilterOperator::HasAny, FilterValue::StringList(list)) => {
            json!({ field_name: { "some": { "name": { "in": list } } } })
        }
        (FilterField::Label, FilterOperator::HasAll, FilterValue::StringList(list)) => {
            json!({ field_name: { "every": { "name": { "in": list } } } })
        }
        (FilterField::Label, FilterOperator::HasNone, FilterValue::StringList(list)) => {
            json!({ field_name: { "none": { "name": { "in": list } } } })
        }
        (FilterField::Label, FilterOperator::IsNull, _) => {
            json!({ field_name: { "every": { "id": { "null": true } } } })
        }
        (FilterField::Label, FilterOperator::IsNotNull, _) => {
            json!({ field_name: { "some": { "id": { "null": false } } } })
        }
        (FilterField::Label, FilterOperator::Equals, FilterValue::String(s)) => {
            json!({ field_name: { "some": { "name": { "eq": s } } } })
        }
        
        // Project operations
        (FilterField::Project, FilterOperator::Equals, FilterValue::String(s)) => {
            json!({ field_name: { "name": { "eq": s } } })
        }
        (FilterField::Project, FilterOperator::IsNull, _) => {
            json!({ field_name: { "null": true } })
        }
        (FilterField::Project, FilterOperator::IsNotNull, _) => {
            json!({ field_name: { "null": false } })
        }
        
        // Date operations
        (FilterField::CreatedAt | FilterField::UpdatedAt | FilterField::DueDate, op, FilterValue::Date(date)) => {
            match op {
                FilterOperator::GreaterThan => json!({ field_name: { "gt": date } }),
                FilterOperator::GreaterThanOrEquals => json!({ field_name: { "gte": date } }),
                FilterOperator::LessThan => json!({ field_name: { "lt": date } }),
                FilterOperator::LessThanOrEquals => json!({ field_name: { "lte": date } }),
                _ => json!({}),
            }
        }
        
        // Default string operations
        (_, FilterOperator::Equals, FilterValue::String(s)) => {
            json!({ field_name: { "eq": s } })
        }
        (_, FilterOperator::NotEquals, FilterValue::String(s)) => {
            json!({ field_name: { "neq": s } })
        }
        (_, FilterOperator::Contains, FilterValue::String(s)) => {
            json!({ field_name: { "containsIgnoreCase": s } })
        }
        
        _ => json!({}),
    }
}

/// Filter errors
#[derive(Debug, thiserror::Error)]
pub enum FilterError {
    #[error("Filter cannot be empty")]
    EmptyFilter,
    
    #[error("Invalid filter combination")]
    InvalidCombination,
    
    #[error("Invalid value for field {field}")]
    InvalidValue { field: String },
}

// Implement conversions for FilterValue
impl From<String> for FilterValue {
    fn from(s: String) -> Self {
        FilterValue::String(s)
    }
}

impl From<&str> for FilterValue {
    fn from(s: &str) -> Self {
        FilterValue::String(s.to_string())
    }
}

impl From<f64> for FilterValue {
    fn from(n: f64) -> Self {
        FilterValue::Number(n)
    }
}

impl From<i32> for FilterValue {
    fn from(n: i32) -> Self {
        FilterValue::Number(n as f64)
    }
}

impl From<u8> for FilterValue {
    fn from(n: u8) -> Self {
        FilterValue::Number(n as f64)
    }
}

impl From<bool> for FilterValue {
    fn from(b: bool) -> Self {
        FilterValue::Boolean(b)
    }
}

impl Default for FilterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_filter() {
        let mut builder = FilterBuilder::new();
        builder.status().not_equals("completed");
        let filter = builder.build().unwrap();
        
        match filter {
            FilterExpression::Group(group) => {
                assert_eq!(group.conditions.len(), 1);
            }
            _ => panic!("Expected group"),
        }
    }
    
    #[test]
    fn test_compound_filter() {
        let mut builder = FilterBuilder::new();
        builder.status().not_equals("completed")
            .and()
            .priority().greater_than(2)
            .and()
            .created_at().within_days(7);
        let filter = builder.build().unwrap();
        
        match filter {
            FilterExpression::Group(group) => {
                assert_eq!(group.conditions.len(), 3);
                assert!(matches!(group.operator, LogicalOperator::And));
            }
            _ => panic!("Expected group"),
        }
    }
    
    #[test]
    fn test_graphql_conversion() {
        let mut builder = FilterBuilder::new();
        builder.title().contains("bug")
            .and()
            .priority().greater_than(2);
        let graphql = builder.to_graphql().unwrap();
        
        assert!(graphql.get("title").is_some());
        assert!(graphql.get("priority").is_some());
    }
}
