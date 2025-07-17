use clap::ArgMatches;
use colored::*;
use crate::client::LinearClient;
use crate::config::{get_api_key, load_config, save_config};
use crate::filtering::{parse_filter_query, build_graphql_filter};
use crate::formatting::issues::print_issues;

pub async fn handle_save_search(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let name = matches.get_one::<String>("name")
        .ok_or("Search name is required")?;
    let query = matches.get_one::<String>("query")
        .ok_or("Search query is required")?;
    
    // Validate the query
    match parse_filter_query(query) {
        Ok(_) => {
            let mut config = load_config();
            config.saved_searches.insert(name.clone(), query.clone());
            save_config(&config)?;
            
            println!("✅ Saved search '{}' successfully!", name);
            println!("Query: {}", query);
            println!("\nRun it with: linear search run {}", name);
        }
        Err(e) => {
            eprintln!("Error: Invalid filter query - {}", e);
            eprintln!("Use 'linear issues --help' to see filter syntax examples");
            return Err(e.into());
        }
    }
    
    Ok(())
}

pub async fn handle_list_searches() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config();
    
    if config.saved_searches.is_empty() {
        println!("No saved searches found.");
        println!("\nSave a search with: linear search save <name> <query>");
    } else {
        println!("Saved searches:");
        println!("{}", "─".repeat(80));
        
        let mut searches: Vec<_> = config.saved_searches.iter().collect();
        searches.sort_by_key(|(name, _)| name.as_str());
        
        for (name, query) in searches {
            println!("\n{} {}", "▸".bright_blue(), name.bright_cyan().bold());
            println!("  Query: {}", query);
            println!("  Run: linear search run {}", name);
        }
    }
    
    Ok(())
}

pub async fn handle_delete_search(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let name = matches.get_one::<String>("name")
        .ok_or("Search name is required")?;
    
    let mut config = load_config();
    
    if config.saved_searches.remove(name).is_some() {
        save_config(&config)?;
        println!("✅ Deleted saved search '{}'", name);
    } else {
        println!("❌ Saved search '{}' not found", name);
    }
    
    Ok(())
}

pub async fn handle_run_search(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let name = matches.get_one::<String>("name")
        .ok_or("Search name is required")?;
    
    let config = load_config();
    let query = config.saved_searches.get(name)
        .ok_or(format!("Saved search '{}' not found", name))?;
    
    println!("Running saved search '{}': {}", name.bright_cyan(), query);
    println!("{}", "─".repeat(80));
    
    // Parse and execute the search
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let format = matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("simple");
    let limit = matches.get_one::<String>("limit")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(50);
    
    match parse_filter_query(query) {
        Ok(filters) => {
            let filter = build_graphql_filter(filters);
            let filter_param = if filter.as_object().unwrap().is_empty() {
                None
            } else {
                Some(filter)
            };
            
            let issues = client.get_issues(filter_param, Some(limit)).await?;
            
            if issues.is_empty() {
                println!("No issues found matching your saved search.");
            } else {
                print_issues(&issues, format);
            }
        }
        Err(e) => {
            eprintln!("Error parsing saved search: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}