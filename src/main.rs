use std::env;
use std::fs;
use std::path::Path;
use std::process;
use colored::*;

use clap::{Arg, ArgMatches, Command};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio;

// Add these to your Cargo.toml:
// [dependencies]
// clap = "4.0"
// reqwest = { version = "0.11", features = ["json"] }
// serde = { version = "1.0", features = ["derive"] }
// serde_json = "1.0"
// tokio = { version = "1.0", features = ["full"] }
// dirs = "5.0"

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";
const CONFIG_FILE: &str = ".linear-cli-config.json";

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    api_key: Option<String>,
    default_team_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
    #[serde(default)]
    path: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Issue {
    id: String,
    identifier: String,
    title: String,
    description: Option<String>,
    url: String,
    priority: Option<u8>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
    state: WorkflowState,
    assignee: Option<User>,
    team: Team,
    labels: LabelConnection,
}

#[derive(Debug, Deserialize, Serialize)]
struct LabelConnection {
    nodes: Vec<Label>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Label {
    id: String,
    name: String,
    color: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkflowState {
    id: String,
    name: String,
    #[serde(rename = "type")]
    state_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct User {
    id: String,
    name: String,
    email: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Team {
    id: String,
    name: String,
    key: String,
}

#[derive(Debug, Deserialize)]
struct Project {
    id: String,
    name: String,
    description: Option<String>,
    url: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    state: String,
    progress: f64,
}

#[derive(Debug, Deserialize)]
struct IssueConnection {
    nodes: Vec<Issue>,
    #[serde(rename = "pageInfo")]
    page_info: PageInfo,
}

#[derive(Debug, Deserialize)]
struct TeamConnection {
    nodes: Vec<Team>,
}

#[derive(Debug, Deserialize)]
struct ProjectConnection {
    nodes: Vec<Project>,
}

#[derive(Debug, Deserialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor")]
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ViewerData {
    viewer: User,
}

#[derive(Debug, Deserialize)]
struct IssuesData {
    issues: IssueConnection,
}

#[derive(Debug, Deserialize)]
struct TeamsData {
    teams: TeamConnection,
}

#[derive(Debug, Deserialize)]
struct ProjectsData {
    projects: ProjectConnection,
}

#[derive(Debug, Deserialize)]
struct IssueCreateData {
    #[serde(rename = "issueCreate")]
    issue_create: IssueCreatePayload,
}

#[derive(Debug, Deserialize)]
struct IssueCreatePayload {
    success: bool,
    issue: Option<Issue>,
}

#[derive(Debug, Deserialize)]
struct ProjectCreateData {
    #[serde(rename = "projectCreate")]
    project_create: ProjectCreatePayload,
}

#[derive(Debug, Deserialize)]
struct ProjectCreatePayload {
    success: bool,
    project: Option<Project>,
}

#[derive(Debug, Deserialize)]
struct IssueUpdateData {
    #[serde(rename = "issueUpdate")]
    issue_update: IssueUpdatePayload,
}

#[derive(Debug, Deserialize)]
struct IssueUpdatePayload {
    success: bool,
    issue: Option<Issue>,
}

#[derive(Debug, Deserialize)]
struct ProjectUpdateData {
    #[serde(rename = "projectUpdate")]
    project_update: ProjectUpdatePayload,
}

#[derive(Debug, Deserialize)]
struct ProjectUpdatePayload {
    success: bool,
    project: Option<Project>,
}

#[derive(Debug, Deserialize)]
struct IssueArchiveData {
    #[serde(rename = "issueArchive")]
    issue_archive: ArchivePayload,
}

#[derive(Debug, Deserialize)]
struct ProjectArchiveData {
    #[serde(rename = "projectArchive")]
    project_archive: ArchivePayload,
}

#[derive(Debug, Deserialize)]
struct ArchivePayload {
    success: bool,
}

struct LinearClient {
    client: reqwest::Client,
    api_key: String,
}

impl LinearClient {
    fn new(api_key: String) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&api_key).expect("Invalid API key format"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_key }
    }

    async fn execute_query<T: for<'de> Deserialize<'de>>(
        &self,
        query: &str,
        variables: Option<Value>,
    ) -> Result<T, Box<dyn std::error::Error>> {
        let body = match variables {
            Some(vars) => json!({ "query": query, "variables": vars }),
            None => json!({ "query": query }),
        };

        let response = self
            .client
            .post(LINEAR_API_URL)
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()).into());
        }

        let graphql_response: GraphQLResponse<T> = response.json().await?;

        if let Some(errors) = graphql_response.errors {
            let error_messages: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
            return Err(format!("GraphQL errors: {}", error_messages.join(", ")).into());
        }

        graphql_response
            .data
            .ok_or("No data returned from GraphQL query".into())
    }

    async fn get_viewer(&self) -> Result<User, Box<dyn std::error::Error>> {
        let query = r#"
            query {
                viewer {
                    id
                    name
                    email
                }
            }
        "#;

        let data: ViewerData = self.execute_query(query, None).await?;
        Ok(data.viewer)
    }

    async fn get_issues(&self, filter: Option<Value>, first: Option<i32>) -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
        let query = r#"
            query($filter: IssueFilter, $first: Int) {
                issues(filter: $filter, first: $first) {
                    nodes {
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
                    }
                    pageInfo {
                        hasNextPage
                        endCursor
                    }
                }
            }
        "#;

        let variables = json!({
            "filter": filter,
            "first": first.unwrap_or(50)
        });

        let data: IssuesData = self.execute_query(query, Some(variables)).await?;
        Ok(data.issues.nodes)
    }

    async fn get_issue_by_identifier(&self, identifier: &str) -> Result<Issue, Box<dyn std::error::Error>> {
        let query = r#"
            query($identifier: String!) {
                issue(id: $identifier) {
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
                }
            }
        "#;

        let variables = json!({
            "identifier": identifier
        });

        #[derive(Debug, Deserialize)]
        struct IssueData {
            issue: Issue,
        }

        let data: IssueData = self.execute_query(query, Some(variables)).await?;
        Ok(data.issue)
    }

    async fn get_teams(&self) -> Result<Vec<Team>, Box<dyn std::error::Error>> {
        let query = r#"
            query {
                teams {
                    nodes {
                        id
                        name
                        key
                    }
                }
            }
        "#;

        let data: TeamsData = self.execute_query(query, None).await?;
        Ok(data.teams.nodes)
    }

    async fn get_projects(&self) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
        let query = r#"
            query {
                projects {
                    nodes {
                        id
                        name
                        description
                        url
                        createdAt
                        state
                        progress
                    }
                }
            }
        "#;

        let data: ProjectsData = self.execute_query(query, None).await?;
        Ok(data.projects.nodes)
    }

    async fn create_issue(
        &self,
        title: &str,
        description: Option<&str>,
        team_id: &str,
        priority: Option<u8>,
        assignee_id: Option<&str>,
        label_ids: Option<Vec<&str>>,
    ) -> Result<Issue, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($input: IssueCreateInput!) {
                issueCreate(input: $input) {
                    success
                    issue {
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
                    }
                }
            }
        "#;

        let mut input = json!({
            "title": title,
            "teamId": team_id
        });

        if let Some(desc) = description {
            input["description"] = json!(desc);
        }
        if let Some(prio) = priority {
            input["priority"] = json!(prio);
        }
        if let Some(assignee) = assignee_id {
            input["assigneeId"] = json!(assignee);
        }
        if let Some(labels) = label_ids {
            input["labelIds"] = json!(labels);
        }

        let variables = json!({ "input": input });

        let data: IssueCreateData = self.execute_query(query, Some(variables)).await?;
        
        if !data.issue_create.success {
            return Err("Failed to create issue".into());
        }

        data.issue_create.issue.ok_or("Issue not returned after creation".into())
    }

    async fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
        team_ids: Option<Vec<&str>>,
    ) -> Result<Project, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($input: ProjectCreateInput!) {
                projectCreate(input: $input) {
                    success
                    project {
                        id
                        name
                        description
                        url
                        createdAt
                        state
                        progress
                    }
                }
            }
        "#;

        let mut input = json!({ "name": name });

        if let Some(desc) = description {
            input["description"] = json!(desc);
        }
        if let Some(teams) = team_ids {
            input["teamIds"] = json!(teams);
        }

        let variables = json!({ "input": input });

        let data: ProjectCreateData = self.execute_query(query, Some(variables)).await?;
        
        if !data.project_create.success {
            return Err("Failed to create project".into());
        }

        data.project_create.project.ok_or("Project not returned after creation".into())
    }

    async fn update_issue(
        &self,
        issue_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        state_id: Option<&str>,
        priority: Option<u8>,
        assignee_id: Option<&str>,
        label_ids: Option<Vec<&str>>,
    ) -> Result<Issue, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($id: String!, $input: IssueUpdateInput!) {
                issueUpdate(id: $id, input: $input) {
                    success
                    issue {
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
                    }
                }
            }
        "#;

        let mut input = json!({});

        if let Some(t) = title {
            input["title"] = json!(t);
        }
        if let Some(desc) = description {
            input["description"] = json!(desc);
        }
        if let Some(state) = state_id {
            input["stateId"] = json!(state);
        }
        if let Some(prio) = priority {
            input["priority"] = json!(prio);
        }
        if let Some(assignee) = assignee_id {
            input["assigneeId"] = json!(assignee);
        }
        if let Some(labels) = label_ids {
            input["labelIds"] = json!(labels);
        }

        let variables = json!({ 
            "id": issue_id,
            "input": input 
        });

        let data: IssueUpdateData = self.execute_query(query, Some(variables)).await?;
        
        if !data.issue_update.success {
            return Err("Failed to update issue".into());
        }

        data.issue_update.issue.ok_or("Issue not returned after update".into())
    }

    async fn update_project(
        &self,
        project_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        state: Option<&str>,
    ) -> Result<Project, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($id: String!, $input: ProjectUpdateInput!) {
                projectUpdate(id: $id, input: $input) {
                    success
                    project {
                        id
                        name
                        description
                        url
                        createdAt
                        state
                        progress
                    }
                }
            }
        "#;

        let mut input = json!({});

        if let Some(n) = name {
            input["name"] = json!(n);
        }
        if let Some(desc) = description {
            input["description"] = json!(desc);
        }
        if let Some(s) = state {
            input["state"] = json!(s);
        }

        let variables = json!({ 
            "id": project_id,
            "input": input 
        });

        let data: ProjectUpdateData = self.execute_query(query, Some(variables)).await?;
        
        if !data.project_update.success {
            return Err("Failed to update project".into());
        }

        data.project_update.project.ok_or("Project not returned after update".into())
    }

    async fn archive_issue(&self, issue_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($id: String!) {
                issueArchive(id: $id) {
                    success
                }
            }
        "#;

        let variables = json!({ "id": issue_id });

        let data: IssueArchiveData = self.execute_query(query, Some(variables)).await?;
        
        Ok(data.issue_archive.success)
    }

    async fn archive_project(&self, project_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($id: String!) {
                projectArchive(id: $id) {
                    success
                }
            }
        "#;

        let variables = json!({ "id": project_id });

        let data: ProjectArchiveData = self.execute_query(query, Some(variables)).await?;
        
        Ok(data.project_archive.success)
    }
}

fn load_config() -> Config {
    let config_path = dirs::home_dir()
        .map(|mut path| {
            path.push(CONFIG_FILE);
            path
        })
        .unwrap_or_else(|| Path::new(CONFIG_FILE).to_path_buf());

    if config_path.exists() {
        let content = fs::read_to_string(config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Config {
            api_key: None,
            default_team_id: None,
        }
    }
}

fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = dirs::home_dir()
        .map(|mut path| {
            path.push(CONFIG_FILE);
            path
        })
        .unwrap_or_else(|| Path::new(CONFIG_FILE).to_path_buf());

    let content = serde_json::to_string_pretty(config)?;
    fs::write(config_path, content)?;
    Ok(())
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: None,
            default_team_id: None,
        }
    }
}

fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    // Try environment variable first
    if let Ok(api_key) = env::var("LINEAR_API_KEY") {
        return Ok(api_key);
    }

    // Then try config file
    let config = load_config();
    if let Some(api_key) = config.api_key {
        return Ok(api_key);
    }

    Err("No API key found. Set LINEAR_API_KEY environment variable or run 'linear auth' to configure.".into())
}

fn print_issues(issues: &[Issue], format: &str) {
    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(issues).unwrap());
        }
        "table" => {
            println!("{:<12} {:<50} {:<15} {:<15} {:<20} {:<10}", 
                     "ID".bold(), 
                     "Title".bold(), 
                     "State".bold(), 
                     "Priority".bold(), 
                     "Assignee".bold(),
                     "Labels".bold());
            println!("{}", "-".repeat(122));
            for issue in issues {
                let assignee = issue.assignee.as_ref()
                    .map(|a| {
                        if a.name.contains('@') {
                            a.name.split('@').next().unwrap_or(&a.name)
                        } else {
                            a.name.split_whitespace().next().unwrap_or(&a.name)
                        }
                    })
                    .unwrap_or("Unassigned");
                let priority = match issue.priority {
                    Some(0) => "None".normal(),
                    Some(1) => "Low".blue(),
                    Some(2) => "Medium".yellow(),
                    Some(3) => "High".bright_red(),
                    Some(4) => "Urgent".red().bold(),
                    _ => "None".normal(),
                };
                let state_color = match issue.state.state_type.as_str() {
                    "backlog" => issue.state.name.white().dimmed(),
                    "unstarted" => issue.state.name.white(),
                    "started" => issue.state.name.yellow(),
                    "completed" => issue.state.name.green(),
                    "canceled" => issue.state.name.red().dimmed(),
                    _ => issue.state.name.normal(),
                };
                let labels = issue.labels.nodes
                    .iter()
                    .map(|l| l.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                
                println!(
                    "{:<12} {:<50} {:<15} {:<15} {:<20} {:<10}",
                    issue.identifier.bright_blue().bold(),
                    truncate(&issue.title, 48),
                    state_color,
                    priority,
                    truncate(assignee, 18),
                    truncate(&labels, 10)
                );
            }
        }
        _ => {
            for issue in issues {
                let state_icon = match issue.state.state_type.as_str() {
                    "backlog" => "⏸",
                    "unstarted" => "○",
                    "started" => "◐",
                    "completed" => "✓",
                    "canceled" => "✗",
                    _ => "•",
                };
                
                let priority_indicator = match issue.priority {
                    Some(4) => " [URGENT]".red().bold(),
                    Some(3) => " [HIGH]".bright_red(),
                    Some(2) => " [MEDIUM]".yellow(),
                    Some(1) => " [LOW]".blue(),
                    _ => "".normal(),
                };
                
                let assignee_text = if let Some(ref assignee) = issue.assignee {
                    let first_name = if assignee.name.contains('@') {
                        // For email addresses, use the part before @
                        assignee.name.split('@').next().unwrap_or(&assignee.name)
                    } else {
                        // For regular names, use the first word
                        assignee.name.split_whitespace().next().unwrap_or(&assignee.name)
                    };
                    format!(" → {}", first_name).bright_black()
                } else {
                    "".normal()
                };
                
                let labels_text = if !issue.labels.nodes.is_empty() {
                    let labels = issue.labels.nodes
                        .iter()
                        .map(|l| l.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!(" [{}]", labels).cyan()
                } else {
                    "".normal()
                };
                
                println!(
                    "{} {} - {}{}{}{}{}",
                    state_icon,
                    issue.identifier.bright_blue().bold(),
                    issue.title,
                    priority_indicator,
                    assignee_text,
                    labels_text,
                    if let Some(ref desc) = issue.description {
                        let cleaned = clean_description(desc);
                        if !cleaned.is_empty() {
                            format!("\n  {}", truncate(&cleaned, 80)).bright_black()
                        } else {
                            "".normal()
                        }
                    } else {
                        "".normal()
                    }
                );
            }
        }
    }
}

fn print_teams(teams: &[Team]) {
    println!("{:<40} {:<20} {:<10}", "ID", "Name", "Key");
    println!("{}", "-".repeat(70));
    for team in teams {
        println!("{:<40} {:<20} {:<10}", team.id, team.name, team.key);
    }
}

fn print_projects(projects: &[Project]) {
    println!("{:<30} {:<15} {:<10} {:<50}", 
             "Name".bold(), 
             "State".bold(), 
             "Progress".bold(),
             "Description".bold());
    println!("{}", "-".repeat(105));
    for project in projects {
        let state_color = match project.state.as_str() {
            "planned" => project.state.blue(),
            "started" => project.state.yellow(),
            "completed" => project.state.green(),
            "canceled" => project.state.red().dimmed(),
            "backlog" => project.state.white().dimmed(),
            _ => project.state.normal(),
        };
        
        let progress_bar = {
            let filled = (project.progress * 10.0) as usize;
            let empty = 10 - filled;
            format!("{}{} {:.0}%",
                    "█".repeat(filled).green(),
                    "░".repeat(empty).bright_black(),
                    project.progress * 100.0)
        };
        
        let description = project.description
            .as_ref()
            .map(|d| {
                let cleaned = clean_description(d);
                if cleaned.is_empty() {
                    "-".bright_black().to_string()
                } else {
                    truncate(&cleaned, 48)
                }
            })
            .unwrap_or_else(|| "-".bright_black().to_string());
        
        println!(
            "{:<30} {:<15} {:<20} {:<50}",
            truncate(&project.name, 28).bold(),
            state_color,
            progress_bar,
            description
        );
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn print_formatted_markdown(text: &str) {
    let mut in_code_block = false;
    let mut code_block_content: Vec<String> = Vec::new();
    let mut consecutive_empty_lines = 0;
    
    for line in text.lines() {
        let trimmed = line.trim();
        
        // Handle empty lines
        if trimmed.is_empty() {
            consecutive_empty_lines += 1;
            if consecutive_empty_lines <= 1 && !in_code_block {
                println!();
            }
            continue;
        } else {
            consecutive_empty_lines = 0;
        }
        
        // Handle code blocks
        if trimmed.starts_with("```") {
            if in_code_block {
                // End of code block - print it
                println!("\n{}", "╭─ Code ─────────────────────────────────────────────────────────────╮".bright_cyan());
                for code_line in &code_block_content {
                    // Use bright_white on default background for better readability
                    let formatted_line = if code_line.trim().is_empty() {
                        format!("{:<68}", "")
                    } else {
                        format!("{:<68}", code_line)
                    };
                    println!("{} {} {}", "│".bright_cyan(), formatted_line.bright_white(), "│".bright_cyan());
                }
                println!("{}", "╰────────────────────────────────────────────────────────────────────╯".bright_cyan());
                println!();
                code_block_content.clear();
                in_code_block = false;
            } else {
                // Start of code block
                in_code_block = true;
            }
            continue;
        }
        
        if in_code_block {
            code_block_content.push(line.to_string());
            continue;
        }
        
        // Handle headers
        if trimmed.starts_with("###") {
            let header = trimmed.trim_start_matches('#').trim();
            println!("\n  {}", header.yellow());
            println!("  {}", "─".repeat(header.len()).yellow());
        } else if trimmed.starts_with("##") {
            let header = trimmed.trim_start_matches('#').trim();
            println!("\n{}", header.bright_yellow().bold());
            println!("{}", "─".repeat(header.len()).bright_yellow());
        } else if trimmed.starts_with("#") {
            let header = trimmed.trim_start_matches('#').trim();
            println!("\n{}", header.bright_cyan().bold());
            println!("{}", "═".repeat(header.len()).bright_cyan());
        } 
        // Handle bullet points
        else if trimmed.starts_with("* ") || trimmed.starts_with("- ") {
            let content = trimmed.trim_start_matches(['*', '-', ' ']);
            let formatted = format_inline_markdown(content);
            println!("  {} {}", "•".bright_green(), formatted);
        }
        // Handle numbered lists
        else if trimmed.chars().next().map_or(false, |c| c.is_numeric()) && 
                (trimmed.starts_with("1.") || trimmed.starts_with("2.") || trimmed.starts_with("3.") || 
                 trimmed.starts_with("4.") || trimmed.starts_with("5.") || trimmed.starts_with("6.")) {
            let parts: Vec<&str> = trimmed.splitn(2, '.').collect();
            if parts.len() == 2 {
                let num = parts[0];
                let content = parts[1].trim();
                let formatted = format_inline_markdown(content);
                println!("  {}. {}", num.bright_cyan(), formatted);
            }
        }
        // Handle indented content (like sub-bullets)
        else if line.starts_with("   ") {
            let formatted = format_inline_markdown(trimmed);
            println!("     {} {}", "◦".bright_black(), formatted);
        }
        // Handle checkboxes
        else if trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            let checked = trimmed.contains("[x]") || trimmed.contains("[X]");
            let content = trimmed.trim_start_matches("- [ ]").trim_start_matches("- [x]").trim_start_matches("- [X]").trim();
            let checkbox = if checked { "☑".green() } else { "☐".bright_black() };
            let formatted = format_inline_markdown(content);
            println!("  {} {}", checkbox, formatted);
        }
        // Regular paragraph
        else {
            let formatted = format_inline_markdown(trimmed);
            println!("{}", formatted);
        }
    }
    
    // Handle any remaining code block content
    if in_code_block && !code_block_content.is_empty() {
        println!("\n{}", "╭─ Code ─────────────────────────────────────────────────────────────╮".bright_cyan());
        for code_line in &code_block_content {
            let formatted_line = if code_line.trim().is_empty() {
                format!("{:<68}", "")
            } else {
                format!("{:<68}", code_line)
            };
            println!("{} {} {}", "│".bright_cyan(), formatted_line.bright_white(), "│".bright_cyan());
        }
        println!("{}", "╰────────────────────────────────────────────────────────────────────╯".bright_cyan());
        println!();
    }
}

fn format_inline_markdown(text: &str) -> String {
    let mut result = text.to_string();
    
    // Handle bold text
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let before = &result[..start];
            let bold_text = &result[start + 2..start + 2 + end];
            let after = &result[start + 2 + end + 2..];
            result = format!("{}{}{}", before, bold_text.bold(), after);
        } else {
            break;
        }
    }
    
    // Handle inline code
    while let Some(start) = result.find('`') {
        if let Some(end) = result[start + 1..].find('`') {
            let before = &result[..start];
            let code_text = &result[start + 1..start + 1 + end];
            let after = &result[start + 1 + end + 1..];
            // Use cyan color for inline code for better readability
            result = format!("{}{}{}", before, code_text.cyan(), after);
        } else {
            break;
        }
    }
    
    // Handle links [text](url) - just show the text part
    while let Some(start) = result.find('[') {
        if let Some(mid) = result[start..].find("](") {
            if let Some(end) = result[start + mid + 2..].find(')') {
                let before = &result[..start];
                let link_text = &result[start + 1..start + mid];
                let after = &result[start + mid + 2 + end + 1..];
                result = format!("{}{}{}", before, link_text.bright_blue(), after);
            } else {
                break;
            }
        } else {
            break;
        }
    }
    
    // Handle emphasis
    result = result.replace("_", "");
    
    result
}

fn print_single_issue(issue: &Issue) {
    println!("\n{}", "─".repeat(80).bright_black());
    
    // Header with ID and title
    println!("{} {} - {}", 
             issue.identifier.bright_blue().bold(),
             "│".bright_black(),
             issue.title.bold());
    
    println!("{}", "─".repeat(80).bright_black());
    
    // Status, Priority, Assignee in one line
    let priority_text = match issue.priority {
        Some(0) => "None".normal(),
        Some(1) => "Low".blue(),
        Some(2) => "Medium".yellow(),
        Some(3) => "High".bright_red(),
        Some(4) => "Urgent".red().bold(),
        _ => "None".normal(),
    };
    
    let state_color = match issue.state.state_type.as_str() {
        "backlog" => issue.state.name.white().dimmed(),
        "unstarted" => issue.state.name.white(),
        "started" => issue.state.name.yellow(),
        "completed" => issue.state.name.green(),
        "canceled" => issue.state.name.red().dimmed(),
        _ => issue.state.name.normal(),
    };
    
    println!("{}: {} {} {}: {} {} {}: {}",
             "Status".bold(),
             state_color,
             "│".bright_black(),
             "Priority".bold(),
             priority_text,
             "│".bright_black(),
             "Team".bold(),
             issue.team.name);
    
    // Assignee
    if let Some(ref assignee) = issue.assignee {
        println!("{}: {}", "Assignee".bold(), assignee.name);
    }
    
    // Labels
    if !issue.labels.nodes.is_empty() {
        let labels = issue.labels.nodes
            .iter()
            .map(|l| format!("{}", l.name.cyan()))
            .collect::<Vec<_>>()
            .join(", ");
        println!("{}: {}", "Labels".bold(), labels);
    }
    
    // Timestamps
    println!("{}: {}", "Created".bold(), issue.created_at.bright_black());
    println!("{}: {}", "Updated".bold(), issue.updated_at.bright_black());
    
    // URL
    println!("{}: {}", "URL".bold(), issue.url.bright_blue());
    
    // Description
    if let Some(ref desc) = issue.description {
        if !desc.trim().is_empty() {
            println!("\n{}", "Description:".bold());
            println!("{}", "─".repeat(80).bright_black());
            print_formatted_markdown(desc);
        }
    }
    
    println!("{}", "─".repeat(80).bright_black());
}

fn clean_description(desc: &str) -> String {
    // Remove markdown headers
    let cleaned = desc
        .lines()
        .map(|line| {
            let line = line.trim();
            if line.starts_with('#') {
                // Remove # headers
                line.trim_start_matches('#').trim()
            } else {
                line
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    
    // Remove other markdown formatting
    let cleaned = cleaned
        .replace("**", "")
        .replace("__", "")
        .replace("```", "")
        .replace("`", "")
        .replace("*", "")
        .replace("_", "")
        .replace("[", "")
        .replace("]", "")
        .replace("(", "")
        .replace(")", "")
        .replace("- ", "")
        .replace("• ", "")
        .replace("  ", " ")
        .trim()
        .to_string();
    
    // Extract first sentence
    if let Some(end) = cleaned.find(|c| c == '.' || c == '!' || c == '?') {
        cleaned[..=end].trim().to_string()
    } else {
        // If no sentence ending found, take the first part up to a comma or semicolon
        if let Some(end) = cleaned.find(|c| c == ',' || c == ';' || c == ':') {
            cleaned[..end].trim().to_string()
        } else {
            cleaned
        }
    }
}

async fn handle_auth(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(api_key) = matches.get_one::<String>("api-key") {
        let mut config = load_config();
        config.api_key = Some(api_key.clone());
        save_config(&config)?;
        println!("API key saved successfully!");
        
        // Test the API key
        let client = LinearClient::new(api_key.clone());
        match client.get_viewer().await {
            Ok(user) => println!("✅ Connected as: {} ({})", user.name, user.email),
            Err(e) => println!("❌ Failed to authenticate: {}", e),
        }
    } else if matches.get_flag("show") {
        let config = load_config();
        match config.api_key {
            Some(key) => println!("API Key: {}...{}", &key[..8], &key[key.len()-4..]),
            None => println!("No API key configured"),
        }
    } else {
        println!("Usage: linear auth --api-key <KEY> or linear auth --show");
    }
    Ok(())
}

async fn handle_issues(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let format = matches.get_one::<String>("format").map(|s| s.as_str()).unwrap_or("simple");
    let limit = matches.get_one::<String>("limit")
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(50);

    let mut filter = json!({});

    // Handle state filters
    if matches.get_flag("todo") || matches.get_flag("backlog") {
        filter["state"] = json!({"type": {"in": ["backlog", "unstarted"]}});
    } else if matches.get_flag("triage") {
        filter["state"] = json!({"type": {"eq": "triage"}});
    } else if matches.get_flag("progress") || matches.get_flag("started") {
        filter["state"] = json!({"type": {"eq": "started"}});
    } else if matches.get_flag("done") || matches.get_flag("completed") {
        filter["state"] = json!({"type": {"eq": "completed"}});
    }

    // Handle assignee filters
    if matches.get_flag("mine") {
        let viewer = client.get_viewer().await?;
        filter["assignee"] = json!({"id": {"eq": viewer.id}});
    } else if let Some(assignee) = matches.get_one::<String>("assignee") {
        filter["assignee"] = json!({"email": {"eq": assignee}});
    }

    // Handle team filter
    if let Some(team) = matches.get_one::<String>("team") {
        filter["team"] = json!({"key": {"eq": team}});
    }

    // Handle search
    if let Some(search) = matches.get_one::<String>("search") {
        filter["title"] = json!({"containsIgnoreCase": search});
    }

    let filter_param = if filter.as_object().unwrap().is_empty() {
        None
    } else {
        Some(filter)
    };

    let issues = client.get_issues(filter_param, Some(limit)).await?;
    
    if issues.is_empty() {
        println!("No issues found matching your criteria.");
    } else {
        println!("Found {} issues:", issues.len());
        print_issues(&issues, format);
    }

    Ok(())
}

async fn handle_create_issue(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let title = matches.get_one::<String>("title")
        .ok_or("Title is required")?;
    let description = matches.get_one::<String>("description");
    
    // Get team ID
    let team_id = if let Some(team_key) = matches.get_one::<String>("team") {
        let teams = client.get_teams().await?;
        teams.iter()
            .find(|t| t.key == *team_key)
            .map(|t| t.id.clone())
            .ok_or(format!("Team '{}' not found", team_key))?
    } else {
        let config = load_config();
        config.default_team_id
            .ok_or("No team specified and no default team configured")?
    };

    let priority = matches.get_one::<String>("priority")
        .and_then(|p| match p.as_str() {
            "none" | "0" => Some(0),
            "low" | "1" => Some(1),
            "medium" | "2" => Some(2),
            "high" | "3" => Some(3),
            "urgent" | "4" => Some(4),
            _ => None,
        });

    let assignee_id = matches.get_one::<String>("assignee");
    let label_ids: Option<Vec<&str>> = matches.get_many::<String>("labels")
        .map(|labels| labels.map(|s| s.as_str()).collect());

    let issue = client.create_issue(
        title,
        description.map(|s| s.as_str()),
        &team_id,
        priority,
        assignee_id.map(|s| s.as_str()),
        label_ids,
    ).await?;

    println!("{} {}", "✅".green(), "Issue created successfully!".green().bold());
    println!("{}: {}", "ID".bold(), issue.identifier.bright_blue().bold());
    println!("{}: {}", "Title".bold(), issue.title);
    println!("{}: {}", "URL".bold(), issue.url.bright_black());
    println!("{}: {}", "Team".bold(), issue.team.name);
    println!("{}: {}", "State".bold(), issue.state.name);

    Ok(())
}

async fn handle_create_project(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let name = matches.get_one::<String>("name")
        .ok_or("Project name is required")?;
    let description = matches.get_one::<String>("description");
    
    let team_ids: Option<Vec<&str>> = matches.get_many::<String>("teams")
        .map(|teams| teams.map(|s| s.as_str()).collect());

    let project = client.create_project(
        name,
        description.map(|s| s.as_str()),
        team_ids,
    ).await?;

    println!("✅ Project created successfully!");
    println!("ID: {}", project.id);
    println!("Name: {}", project.name);
    println!("URL: {}", project.url);

    Ok(())
}

async fn handle_issue(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);
    
    let identifier = matches.get_one::<String>("identifier")
        .ok_or("Issue identifier is required")?;
    
    let issue = client.get_issue_by_identifier(identifier).await?;
    print_single_issue(&issue);
    
    Ok(())
}

async fn handle_update_issue(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let issue_id = matches.get_one::<String>("id")
        .ok_or("Issue ID is required")?;
    
    let title = matches.get_one::<String>("title");
    let description = matches.get_one::<String>("description");
    let state_id = matches.get_one::<String>("state");
    let priority = matches.get_one::<String>("priority")
        .and_then(|p| match p.as_str() {
            "none" | "0" => Some(0),
            "low" | "1" => Some(1),
            "medium" | "2" => Some(2),
            "high" | "3" => Some(3),
            "urgent" | "4" => Some(4),
            _ => None,
        });
    let assignee_id = matches.get_one::<String>("assignee");
    let label_ids: Option<Vec<&str>> = matches.get_many::<String>("labels")
        .map(|labels| labels.map(|s| s.as_str()).collect());

    // Check if at least one field is being updated
    if title.is_none() && description.is_none() && state_id.is_none() && 
       priority.is_none() && assignee_id.is_none() && label_ids.is_none() {
        return Err("No fields to update. Provide at least one field to update.".into());
    }

    let issue = client.update_issue(
        issue_id,
        title.map(|s| s.as_str()),
        description.map(|s| s.as_str()),
        state_id.map(|s| s.as_str()),
        priority,
        assignee_id.map(|s| s.as_str()),
        label_ids,
    ).await?;

    println!("{} {}", "✅".green(), "Issue updated successfully!".green().bold());
    println!("{}: {}", "ID".bold(), issue.identifier.bright_blue().bold());
    println!("{}: {}", "Title".bold(), issue.title);
    println!("{}: {}", "URL".bold(), issue.url.bright_black());
    println!("{}: {}", "State".bold(), issue.state.name);

    Ok(())
}

async fn handle_update_project(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let project_id = matches.get_one::<String>("id")
        .ok_or("Project ID is required")?;
    
    let name = matches.get_one::<String>("name");
    let description = matches.get_one::<String>("description");
    let state = matches.get_one::<String>("state");

    // Check if at least one field is being updated
    if name.is_none() && description.is_none() && state.is_none() {
        return Err("No fields to update. Provide at least one field to update.".into());
    }

    let project = client.update_project(
        project_id,
        name.map(|s| s.as_str()),
        description.map(|s| s.as_str()),
        state.map(|s| s.as_str()),
    ).await?;

    println!("✅ Project updated successfully!");
    println!("ID: {}", project.id);
    println!("Name: {}", project.name);
    println!("URL: {}", project.url);
    println!("State: {}", project.state);

    Ok(())
}

async fn handle_delete_issue(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let issue_id = matches.get_one::<String>("id")
        .ok_or("Issue ID is required")?;
    
    let success = client.archive_issue(issue_id).await?;
    
    if success {
        println!("✅ Issue archived successfully!");
        println!("Issue ID: {}", issue_id);
    } else {
        return Err("Failed to archive issue".into());
    }

    Ok(())
}

async fn handle_delete_project(matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let project_id = matches.get_one::<String>("id")
        .ok_or("Project ID is required")?;
    
    let success = client.archive_project(project_id).await?;
    
    if success {
        println!("✅ Project archived successfully!");
        println!("Project ID: {}", project_id);
    } else {
        return Err("Failed to archive project".into());
    }

    Ok(())
}

async fn handle_teams(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let teams = client.get_teams().await?;
    
    if teams.is_empty() {
        println!("No teams found.");
    } else {
        println!("Found {} teams:", teams.len());
        print_teams(&teams);
    }

    Ok(())
}

async fn handle_projects(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let projects = client.get_projects().await?;
    
    if projects.is_empty() {
        println!("No projects found.");
    } else {
        println!("Found {} projects:", projects.len());
        print_projects(&projects);
    }

    Ok(())
}

async fn handle_whoami(_matches: &ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let api_key = get_api_key()?;
    let client = LinearClient::new(api_key);

    let user = client.get_viewer().await?;
    println!("Logged in as: {} ({})", user.name, user.email);
    println!("User ID: {}", user.id);

    Ok(())
}

#[tokio::main]
async fn main() {
    let app = Command::new("linear")
        .about("Linear CLI - Interact with Linear's API from the command line")
        .version("1.0.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("auth")
                .about("Authenticate with Linear")
                .arg(
                    Arg::new("api-key")
                        .long("api-key")
                        .value_name("KEY")
                        .help("Set your Linear API key")
                        .required(false)
                )
                .arg(
                    Arg::new("show")
                        .long("show")
                        .help("Show current API key")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("issues")
                .about("List and filter issues")
                .arg(
                    Arg::new("todo")
                        .long("todo")
                        .help("Show todo/backlog issues")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("backlog")
                        .long("backlog")
                        .help("Show backlog issues")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("triage")
                        .long("triage")
                        .help("Show issues in triage")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("progress")
                        .long("progress")
                        .help("Show issues in progress")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("started")
                        .long("started")
                        .help("Show started issues")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("done")
                        .long("done")
                        .help("Show completed issues")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("completed")
                        .long("completed")
                        .help("Show completed issues")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("mine")
                        .long("mine")
                        .help("Show issues assigned to me")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("assignee")
                        .long("assignee")
                        .value_name("EMAIL")
                        .help("Filter by assignee email")
                )
                .arg(
                    Arg::new("team")
                        .long("team")
                        .value_name("TEAM_KEY")
                        .help("Filter by team key")
                )
                .arg(
                    Arg::new("search")
                        .long("search")
                        .short('s')
                        .value_name("QUERY")
                        .help("Search in issue titles")
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .short('f')
                        .value_name("FORMAT")
                        .help("Output format: simple, table, json")
                        .default_value("simple")
                )
                .arg(
                    Arg::new("limit")
                        .long("limit")
                        .short('l')
                        .value_name("NUMBER")
                        .help("Limit number of results")
                        .default_value("50")
                )
        )
        .subcommand(
            Command::new("create")
                .about("Create Linear resources")
                .subcommand_required(true)
                .subcommand(
                    Command::new("issue")
                        .about("Create a new issue")
                        .arg(
                            Arg::new("title")
                                .value_name("TITLE")
                                .help("Issue title")
                                .required(true)
                                .index(1)
                        )
                        .arg(
                            Arg::new("description")
                                .value_name("DESCRIPTION")
                                .help("Issue description")
                                .index(2)
                        )
                        .arg(
                            Arg::new("team")
                                .long("team")
                                .short('t')
                                .value_name("TEAM_KEY")
                                .help("Team key (e.g., ENG)")
                        )
                        .arg(
                            Arg::new("priority")
                                .long("priority")
                                .short('p')
                                .value_name("PRIORITY")
                                .help("Priority: none/0, low/1, medium/2, high/3, urgent/4")
                        )
                        .arg(
                            Arg::new("assignee")
                                .long("assignee")
                                .short('a')
                                .value_name("USER_ID")
                                .help("Assignee user ID")
                        )
                        .arg(
                            Arg::new("labels")
                                .long("labels")
                                .short('l')
                                .value_name("LABEL_ID")
                                .help("Label IDs")
                                .action(clap::ArgAction::Append)
                        )
                )
                .subcommand(
                    Command::new("project")
                        .about("Create a new project")
                        .arg(
                            Arg::new("name")
                                .value_name("NAME")
                                .help("Project name")
                                .required(true)
                                .index(1)
                        )
                        .arg(
                            Arg::new("description")
                                .value_name("DESCRIPTION")
                                .help("Project description")
                                .index(2)
                        )
                        .arg(
                            Arg::new("teams")
                                .long("teams")
                                .short('t')
                                .value_name("TEAM_ID")
                                .help("Team IDs")
                                .action(clap::ArgAction::Append)
                        )
                )
        )
        .subcommand(
            Command::new("update")
                .about("Update Linear resources")
                .subcommand_required(true)
                .subcommand(
                    Command::new("issue")
                        .about("Update an existing issue")
                        .arg(
                            Arg::new("id")
                                .value_name("ISSUE_ID")
                                .help("Issue ID to update")
                                .required(true)
                                .index(1)
                        )
                        .arg(
                            Arg::new("title")
                                .long("title")
                                .short('t')
                                .value_name("TITLE")
                                .help("New issue title")
                        )
                        .arg(
                            Arg::new("description")
                                .long("description")
                                .short('d')
                                .value_name("DESCRIPTION")
                                .help("New issue description")
                        )
                        .arg(
                            Arg::new("state")
                                .long("state")
                                .short('s')
                                .value_name("STATE_ID")
                                .help("New state ID")
                        )
                        .arg(
                            Arg::new("priority")
                                .long("priority")
                                .short('p')
                                .value_name("PRIORITY")
                                .help("Priority: none/0, low/1, medium/2, high/3, urgent/4")
                        )
                        .arg(
                            Arg::new("assignee")
                                .long("assignee")
                                .short('a')
                                .value_name("USER_ID")
                                .help("New assignee user ID")
                        )
                        .arg(
                            Arg::new("labels")
                                .long("labels")
                                .short('l')
                                .value_name("LABEL_ID")
                                .help("New label IDs")
                                .action(clap::ArgAction::Append)
                        )
                )
                .subcommand(
                    Command::new("project")
                        .about("Update an existing project")
                        .arg(
                            Arg::new("id")
                                .value_name("PROJECT_ID")
                                .help("Project ID to update")
                                .required(true)
                                .index(1)
                        )
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .short('n')
                                .value_name("NAME")
                                .help("New project name")
                        )
                        .arg(
                            Arg::new("description")
                                .long("description")
                                .short('d')
                                .value_name("DESCRIPTION")
                                .help("New project description")
                        )
                        .arg(
                            Arg::new("state")
                                .long("state")
                                .short('s')
                                .value_name("STATE")
                                .help("New project state")
                        )
                )
        )
        .subcommand(
            Command::new("delete")
                .about("Delete (archive) Linear resources")
                .subcommand_required(true)
                .subcommand(
                    Command::new("issue")
                        .about("Archive an issue")
                        .arg(
                            Arg::new("id")
                                .value_name("ISSUE_ID")
                                .help("Issue ID to archive")
                                .required(true)
                                .index(1)
                        )
                )
                .subcommand(
                    Command::new("project")
                        .about("Archive a project")
                        .arg(
                            Arg::new("id")
                                .value_name("PROJECT_ID")
                                .help("Project ID to archive")
                                .required(true)
                                .index(1)
                        )
                )
        )
        .subcommand(
            Command::new("teams")
                .about("List teams")
        )
        .subcommand(
            Command::new("projects")
                .about("List projects")
        )
        .subcommand(
            Command::new("whoami")
                .about("Show current user information")
        )
        .subcommand(
            Command::new("issue")
                .about("View a single issue with full details")
                .arg(
                    Arg::new("identifier")
                        .value_name("ISSUE_ID")
                        .help("Issue identifier (e.g., INF-31)")
                        .required(true)
                        .index(1)
                )
        );

    let matches = app.get_matches();

    let result = match matches.subcommand() {
        Some(("auth", sub_matches)) => handle_auth(sub_matches).await,
        Some(("issues", sub_matches)) => handle_issues(sub_matches).await,
        Some(("create", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("issue", issue_matches)) => handle_create_issue(issue_matches).await,
                Some(("project", project_matches)) => handle_create_project(project_matches).await,
                _ => {
                    eprintln!("Unknown create subcommand. Use 'linear create --help' for available options.");
                    process::exit(1);
                }
            }
        }
        Some(("update", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("issue", issue_matches)) => handle_update_issue(issue_matches).await,
                Some(("project", project_matches)) => handle_update_project(project_matches).await,
                _ => {
                    eprintln!("Unknown update subcommand. Use 'linear update --help' for available options.");
                    process::exit(1);
                }
            }
        }
        Some(("delete", sub_matches)) => {
            match sub_matches.subcommand() {
                Some(("issue", issue_matches)) => handle_delete_issue(issue_matches).await,
                Some(("project", project_matches)) => handle_delete_project(project_matches).await,
                _ => {
                    eprintln!("Unknown delete subcommand. Use 'linear delete --help' for available options.");
                    process::exit(1);
                }
            }
        }
        Some(("teams", sub_matches)) => handle_teams(sub_matches).await,
        Some(("projects", sub_matches)) => handle_projects(sub_matches).await,
        Some(("whoami", sub_matches)) => handle_whoami(sub_matches).await,
        Some(("issue", sub_matches)) => handle_issue(sub_matches).await,
        _ => {
            eprintln!("Unknown command. Use 'linear --help' for available commands.");
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}