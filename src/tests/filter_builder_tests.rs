use crate::filtering::builder::FilterBuilder;
use crate::filtering::parser::parse_filter;

#[test]
fn test_filter_builder_simple() {
    let mut builder = FilterBuilder::new();
    builder.status().equals("In Progress");
    
    let graphql = builder.to_graphql();
    assert!(graphql.is_ok());
    let graphql_str = format!("{:?}", graphql.unwrap());
    assert!(graphql_str.contains("state"));
    assert!(graphql_str.contains("name"));
    assert!(graphql_str.contains("eq"));
    assert!(graphql_str.contains("In Progress"));
}

#[test]
fn test_filter_builder_compound() {
    let mut builder = FilterBuilder::new();
    builder.status().not_equals("Done")
        .and()
        .priority().greater_than(2);
    
    let graphql = builder.to_graphql();
    assert!(graphql.is_ok());
    let graphql_str = format!("{:?}", graphql.unwrap());
    assert!(graphql_str.contains("and"));
}

#[test]
fn test_filter_builder_with_labels() {
    let mut builder = FilterBuilder::new();
    builder.label().contains("bug")
        .or()
        .label().contains("critical");
    
    let graphql = builder.to_graphql();
    assert!(graphql.is_ok());
    let graphql_str = format!("{:?}", graphql.unwrap());
    assert!(graphql_str.contains("or"));
    assert!(graphql_str.contains("labels"));
}

#[test]
fn test_filter_parser_simple() {
    let result = parse_filter("status:done");
    assert!(result.is_ok());
    
    let result = parse_filter("priority:>2");
    assert!(result.is_ok());
    
    let result = parse_filter("assignee:john");
    assert!(result.is_ok());
}

#[test]
fn test_filter_parser_compound() {
    let result = parse_filter("status:done AND priority:>2");
    assert!(result.is_ok());
    
    let result = parse_filter("label:bug OR label:feature");
    assert!(result.is_ok());
}

#[test]
fn test_filter_parser_quoted_values() {
    let result = parse_filter("title:\"my important task\"");
    assert!(result.is_ok());
    
    let result = parse_filter("status:\"In Progress\"");
    assert!(result.is_ok());
}

#[test]
fn test_filter_parser_relative_dates() {
    let result = parse_filter("created:>7d");
    assert!(result.is_ok());
    
    let result = parse_filter("updated:<2w");
    assert!(result.is_ok());
}

#[test]
fn test_filter_parser_negation() {
    let result = parse_filter("NOT status:done");
    assert!(result.is_ok());
    
    let result = parse_filter("status:!done");
    assert!(result.is_ok());
}