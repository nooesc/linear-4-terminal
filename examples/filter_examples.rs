use linear_cli::filtering::{FilterBuilder, FilterAdapter, parse_filter};
use serde_json::json;

fn main() {
    println!("Linear CLI Filter System Examples\n");
    
    // Example 1: Simple filter using the builder
    println!("Example 1: Builder API - Open high-priority issues");
    let filter1 = FilterBuilder::new()
        .status().not_equals("completed")
        .and()
        .priority().greater_than(2)
        .build()
        .unwrap();
    
    match filter1.to_graphql() {
        Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
        Err(e) => println!("Error: {}\n", e),
    }
    
    // Example 2: Date filtering
    println!("Example 2: Builder API - Recently created issues");
    let filter2 = FilterBuilder::new()
        .created_at().within_days(7)
        .and()
        .status().not_equals("completed")
        .build()
        .unwrap();
    
    match filter2.to_graphql() {
        Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
        Err(e) => println!("Error: {}\n", e),
    }
    
    // Example 3: Complex filter with labels
    println!("Example 3: Builder API - Bugs without assignee");
    let filter3 = FilterBuilder::new()
        .label().equals("bug")
        .and()
        .assignee().is_null()
        .and()
        .status().not_equals("completed")
        .build()
        .unwrap();
    
    match filter3.to_graphql() {
        Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
        Err(e) => println!("Error: {}\n", e),
    }
    
    // Example 4: Parser - Simple query
    println!("Example 4: Parser - Simple query");
    match parse_filter("status:completed AND priority>2") {
        Ok(builder) => {
            match builder.to_graphql() {
                Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
                Err(e) => println!("Error: {}\n", e),
            }
        }
        Err(e) => println!("Parse Error: {}\n", e),
    }
    
    // Example 5: Parser - Complex query with dates
    println!("Example 5: Parser - Complex date query");
    match parse_filter("created>7d AND updated<2d AND status!=completed") {
        Ok(builder) => {
            match builder.to_graphql() {
                Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
                Err(e) => println!("Error: {}\n", e),
            }
        }
        Err(e) => println!("Parse Error: {}\n", e),
    }
    
    // Example 6: Parser - Quoted values and operators
    println!("Example 6: Parser - String operators");
    match parse_filter(r#"title~"bug fix" AND description^=TODO"#) {
        Ok(builder) => {
            match builder.to_graphql() {
                Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
                Err(e) => println!("Error: {}\n", e),
            }
        }
        Err(e) => println!("Parse Error: {}\n", e),
    }
    
    // Example 7: Adapter - Legacy compatibility
    println!("Example 7: Adapter - Legacy and new syntax");
    match FilterAdapter::parse_and_build("status!=completed AND priority>2 AND created>7d") {
        Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
        Err(e) => println!("Error: {}\n", e),
    }
    
    // Example 8: OR operations
    println!("Example 8: Builder API - OR operations");
    let filter8 = FilterBuilder::new()
        .status().equals("backlog")
        .or()
        .status().equals("unstarted")
        .and()
        .priority().greater_than_or_equals(3)
        .build()
        .unwrap();
    
    match filter8.to_graphql() {
        Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
        Err(e) => println!("Error: {}\n", e),
    }
    
    // Example 9: List operations
    println!("Example 9: Parser - IN operator");
    match parse_filter("status in:backlog,unstarted,started") {
        Ok(builder) => {
            match builder.to_graphql() {
                Ok(graphql) => println!("GraphQL: {}\n", serde_json::to_string_pretty(&graphql).unwrap()),
                Err(e) => println!("Error: {}\n", e),
            }
        }
        Err(e) => println!("Parse Error: {}\n", e),
    }
}