use linear_cli::filtering::{FilterBuilder, FilterAdapter, parse_filter};

#[test]
fn test_builder_simple_filter() {
    let mut builder = FilterBuilder::new();
    builder.field(linear_cli::filtering::FilterField::Status)
        .not_equals("completed");
    
    let graphql = builder.to_graphql().unwrap();
    assert!(graphql.get("state").is_some());
}

#[test]
fn test_builder_compound_filter() {
    let mut builder = FilterBuilder::new();
    builder.field(linear_cli::filtering::FilterField::Status)
        .not_equals("completed")
        .and()
        .field(linear_cli::filtering::FilterField::Priority)
        .greater_than(2);
    
    let graphql = builder.to_graphql().unwrap();
    assert!(graphql.get("state").is_some());
    assert!(graphql.get("priority").is_some());
}

#[test]
fn test_parser_simple() {
    let builder = parse_filter("status:completed").unwrap();
    let graphql = builder.to_graphql().unwrap();
    assert!(graphql.get("state").is_some());
}

#[test]
fn test_parser_compound() {
    let builder = parse_filter("status!=completed AND priority>2").unwrap();
    let graphql = builder.to_graphql().unwrap();
    assert!(graphql.get("state").is_some());
    assert!(graphql.get("priority").is_some());
}

#[test]
fn test_parser_relative_dates() {
    let builder = parse_filter("created>7d").unwrap();
    let graphql = builder.to_graphql().unwrap();
    assert!(graphql.get("createdAt").is_some());
}

#[test]
fn test_adapter_backward_compatibility() {
    // Test that the adapter can handle both old and new syntax
    let result1 = FilterAdapter::parse_and_build("status:completed").unwrap();
    assert!(result1.get("state").is_some());
    
    let result2 = FilterAdapter::parse_and_build("status!=completed AND priority>2").unwrap();
    assert!(result2.get("state").is_some());
    assert!(result2.get("priority").is_some());
}

#[test]
fn test_quoted_values() {
    let builder = parse_filter(r#"title~"bug fix""#).unwrap();
    let graphql = builder.to_graphql().unwrap();
    let title_filter = graphql.get("title").unwrap();
    assert!(title_filter.get("containsIgnoreCase").is_some());
}

#[test]
fn test_list_operator() {
    let builder = parse_filter("status in:backlog,unstarted,started").unwrap();
    let graphql = builder.to_graphql().unwrap();
    let state_filter = graphql.get("state").unwrap();
    let name_filter = state_filter.get("name").unwrap();
    assert!(name_filter.get("in").is_some());
}

#[test]
fn test_null_checks() {
    let builder = parse_filter("assignee:null").unwrap();
    let graphql = builder.to_graphql().unwrap();
    let assignee_filter = graphql.get("assignee").unwrap();
    assert_eq!(assignee_filter.get("null").unwrap(), &true);
}

#[test]
fn test_label_filters() {
    let mut builder = FilterBuilder::new();
    builder.field(linear_cli::filtering::FilterField::Label)
        .equals("bug");
    
    let graphql = builder.to_graphql().unwrap();
    let labels_filter = graphql.get("labels").unwrap();
    assert!(labels_filter.get("some").is_some());
}