use serde_json::Value;

use super::builder::FilterBuilder;
use super::parser::parse_filter;
use super::query::{FilterQuery, FilterOperator as LegacyOperator, parse_filter_query as legacy_parse, build_graphql_filter as legacy_build};

/// Adapter to use the new filter system with the existing API
pub struct FilterAdapter;

impl FilterAdapter {
    /// Parse a filter query string and return GraphQL filter JSON
    /// This provides a drop-in replacement for the existing parse + build workflow
    pub fn parse_and_build(query: &str) -> Result<Value, String> {
        // First, try the new parser
        match parse_filter(query) {
            Ok(builder) => {
                builder.to_graphql()
                    .map_err(|e| format!("Filter build error: {}", e))
            }
            Err(e) => {
                // Fall back to legacy parser for backward compatibility
                
                let filters = legacy_parse(query)?;
                Ok(legacy_build(filters))
            }
        }
    }
    
    /// Convert a legacy FilterQuery to the new FilterBuilder
    pub fn from_legacy(queries: Vec<FilterQuery>) -> Result<FilterBuilder, String> {
        let mut builder = FilterBuilder::new();
        
        for (i, query) in queries.iter().enumerate() {
            if i > 0 {
                builder.and(); // Default to AND for multiple conditions
            }
            
            // Map field names
            let field = match query.field.as_str() {
                "assignee" => builder.assignee(),
                "state" | "status" => builder.status(),
                "priority" => builder.priority(),
                "title" => builder.title(),
                "description" => builder.description(),
                "label" => builder.label(),
                "created" => builder.created_at(),
                "updated" => builder.updated_at(),
                _ => builder.field(super::builder::FilterField::Custom(query.field.clone())),
            };
            
            // Map operators and apply conditions
            match (&query.operator, &query.value) {
                (LegacyOperator::Equals, v) => { field.equals(v.as_str()); }
                (LegacyOperator::NotEquals, v) => { field.not_equals(v.as_str()); }
                (LegacyOperator::GreaterThan, v) => {
                    if let Ok(n) = v.parse::<f64>() {
                        field.greater_than(n);
                    } else {
                        field.greater_than(v.as_str());
                    }
                }
                (LegacyOperator::LessThan, v) => {
                    if let Ok(n) = v.parse::<f64>() {
                        field.less_than(n);
                    } else {
                        field.less_than(v.as_str());
                    }
                }
                (LegacyOperator::Contains, v) => { field.contains(v.as_str()); }
                (LegacyOperator::In, v) => {
                    let values: Vec<String> = v.split(',').map(|s| s.trim().to_string()).collect();
                    field.in_list(values);
                }
                (LegacyOperator::HasAssignee, _) => { field.is_not_null(); }
                (LegacyOperator::NoAssignee, _) => { field.is_null(); }
                (LegacyOperator::HasLabel, v) => { field.equals(v.as_str()); }
                (LegacyOperator::NoLabel, _) => { field.is_null(); }
            }
        }
        
        Ok(builder)
    }
}

/// Helper function to provide examples of the new filter syntax
pub fn print_filter_examples() {
    println!("Linear CLI Filter Syntax Examples:");
    println!("==================================");
    println!();
    println!("Basic filters:");
    println!("  status:completed                    # Issues with completed status");
    println!("  status!=completed                   # Issues not completed");
    println!("  priority>2                          # High priority issues (3+)");
    println!("  assignee=john@example.com          # Issues assigned to John");
    println!("  title~bug                          # Issues with 'bug' in title");
    println!();
    println!("Compound filters:");
    println!("  status!=completed AND priority>2    # Open high-priority issues");
    println!("  status:started OR status:unstarted  # Active issues");
    println!("  (priority>2 OR label:urgent) AND status!=completed");
    println!();
    println!("Date filters:");
    println!("  created>7d                         # Created in last 7 days");
    println!("  updated<2w                         # Not updated for 2 weeks");
    println!("  created>1m AND updated<1w          # Old but recently updated");
    println!();
    println!("String operators:");
    println!("  title~\"bug fix\"                   # Contains 'bug fix'");
    println!("  title^=Feature                     # Starts with 'Feature'");
    println!("  description$=TODO                  # Ends with 'TODO'");
    println!();
    println!("List operators:");
    println!("  status in:backlog,unstarted        # Multiple statuses");
    println!("  label in:bug,critical,urgent       # Has any of these labels");
    println!();
    println!("Null checks:");
    println!("  assignee:null                      # Unassigned issues");
    println!("  project!=null                      # Issues with a project");
    println!();
    println!("Negation:");
    println!("  NOT status:completed               # Not completed");
    println!("  NOT (priority<2 OR assignee:null) # Assigned important issues");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_adapter_basic() {
        let result = FilterAdapter::parse_and_build("status:completed").unwrap();
        assert!(result.get("state").is_some());
    }
    
    #[test]
    fn test_adapter_complex() {
        let result = FilterAdapter::parse_and_build("status!=completed AND priority>2 AND created>7d").unwrap();
        assert!(result.get("state").is_some());
        assert!(result.get("priority").is_some());
        assert!(result.get("createdAt").is_some());
    }
}