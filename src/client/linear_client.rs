use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::constants::{COMMENT_FIELDS, ISSUE_FIELDS, LINEAR_API_URL, PROJECT_FIELDS};
use crate::models::*;

pub struct LinearClient {
    client: reqwest::Client,
}

impl LinearClient {
    pub fn new(api_key: String) -> Self {
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

        Self { client }
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

    pub async fn get_viewer(&self) -> Result<User, Box<dyn std::error::Error>> {
        let query = r#"
            query {
                viewer {
                    id
                    name
                    email
                }
            }
        "#;

        let data: graphql::ViewerData = self.execute_query(query, None).await?;
        Ok(data.viewer)
    }

    pub async fn get_issues(&self, filter: Option<Value>, first: Option<i32>) -> Result<Vec<Issue>, Box<dyn std::error::Error>> {
        let query = format!(r#"
            query($filter: IssueFilter, $first: Int) {{
                issues(filter: $filter, first: $first) {{
                    nodes {{{}}}
                }}
            }}
        "#, ISSUE_FIELDS);

        let variables = json!({
            "filter": filter,
            "first": first.unwrap_or(50)
        });

        let data: graphql::IssuesData = self.execute_query(&query, Some(variables)).await?;
        Ok(data.issues.nodes)
    }

    pub async fn get_issue_by_identifier(&self, identifier: &str) -> Result<Issue, Box<dyn std::error::Error>> {
        let query = format!(r#"
            query($identifier: String!) {{
                issue(id: $identifier) {{{}}}
            }}
        "#, ISSUE_FIELDS);

        let variables = json!({
            "identifier": identifier
        });

        #[derive(Debug, Deserialize)]
        struct IssueData {
            issue: Issue,
        }

        let data: IssueData = self.execute_query(&query, Some(variables)).await?;
        Ok(data.issue)
    }

    pub async fn get_teams(&self) -> Result<Vec<Team>, Box<dyn std::error::Error>> {
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

        let data: graphql::TeamsData = self.execute_query(query, None).await?;
        Ok(data.teams.nodes)
    }

    pub async fn get_projects(&self) -> Result<Vec<Project>, Box<dyn std::error::Error>> {
        let query = format!(r#"
            query {{
                projects {{
                    nodes {{{}}}
                }}
            }}
        "#, PROJECT_FIELDS);

        let data: graphql::ProjectsData = self.execute_query(&query, None).await?;
        Ok(data.projects.nodes)
    }

    pub async fn create_issue(
        &self,
        title: &str,
        description: Option<&str>,
        team_id: &str,
        priority: Option<u8>,
        assignee_id: Option<&str>,
        label_ids: Option<Vec<&str>>,
    ) -> Result<Issue, Box<dyn std::error::Error>> {
        let query = format!(r#"
            mutation($input: IssueCreateInput!) {{
                issueCreate(input: $input) {{
                    success
                    issue {{{}}}
                }}
            }}
        "#, ISSUE_FIELDS);

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

        let data: graphql::IssueCreateData = self.execute_query(&query, Some(variables)).await?;
        Self::check_success(data.issue_create.success, data.issue_create.issue, "Failed to create issue")
    }
    
    fn check_success<T>(success: bool, data: Option<T>, error_msg: &str) -> Result<T, Box<dyn std::error::Error>> {
        if !success {
            return Err(error_msg.into());
        }
        data.ok_or_else(|| format!("{} but no data returned", error_msg).into())
    }

    pub async fn create_project(
        &self,
        name: &str,
        description: Option<&str>,
        team_ids: Option<Vec<&str>>,
    ) -> Result<Project, Box<dyn std::error::Error>> {
        let query = format!(r#"
            mutation($input: ProjectCreateInput!) {{
                projectCreate(input: $input) {{
                    success
                    project {{{}}}
                }}
            }}
        "#, PROJECT_FIELDS);

        let mut input = json!({ "name": name });

        if let Some(desc) = description {
            input["description"] = json!(desc);
        }
        if let Some(teams) = team_ids {
            input["teamIds"] = json!(teams);
        }

        let variables = json!({ "input": input });

        let data: graphql::ProjectCreateData = self.execute_query(&query, Some(variables)).await?;
        Self::check_success(data.project_create.success, data.project_create.project, "Failed to create project")
    }

    pub async fn update_issue(
        &self,
        issue_id: &str,
        title: Option<&str>,
        description: Option<&str>,
        state_id: Option<&str>,
        priority: Option<u8>,
        assignee_id: Option<&str>,
        label_ids: Option<Vec<&str>>,
    ) -> Result<Issue, Box<dyn std::error::Error>> {
        let query = format!(r#"
            mutation($id: String!, $input: IssueUpdateInput!) {{
                issueUpdate(id: $id, input: $input) {{
                    success
                    issue {{{}}}
                }}
            }}
        "#, ISSUE_FIELDS);

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

        let data: graphql::IssueUpdateData = self.execute_query(&query, Some(variables)).await?;
        Self::check_success(data.issue_update.success, data.issue_update.issue, "Failed to update issue")
    }

    pub async fn update_project(
        &self,
        project_id: &str,
        name: Option<&str>,
        description: Option<&str>,
        state: Option<&str>,
    ) -> Result<Project, Box<dyn std::error::Error>> {
        let query = format!(r#"
            mutation($id: String!, $input: ProjectUpdateInput!) {{
                projectUpdate(id: $id, input: $input) {{
                    success
                    project {{{}}}
                }}
            }}
        "#, PROJECT_FIELDS);

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

        let data: graphql::ProjectUpdateData = self.execute_query(&query, Some(variables)).await?;
        Self::check_success(data.project_update.success, data.project_update.project, "Failed to update project")
    }

    pub async fn archive_issue(&self, issue_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($id: String!) {
                issueArchive(id: $id) {
                    success
                }
            }
        "#;

        let variables = json!({ "id": issue_id });

        let data: graphql::IssueArchiveData = self.execute_query(query, Some(variables)).await?;
        
        Ok(data.issue_archive.success)
    }

    pub async fn archive_project(&self, project_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($id: String!) {
                projectArchive(id: $id) {
                    success
                }
            }
        "#;

        let variables = json!({ "id": project_id });

        let data: graphql::ProjectArchiveData = self.execute_query(query, Some(variables)).await?;
        
        Ok(data.project_archive.success)
    }

    pub async fn get_comments(&self, issue_id: &str) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
        let query = format!(
            r#"
            query($issueId: String!) {{
                issue(id: $issueId) {{
                    comments {{
                        nodes {{
                            {}
                        }}
                    }}
                }}
            }}
            "#,
            COMMENT_FIELDS
        );
        let variables = json!({ "issueId": issue_id });
        
        #[derive(Debug, Deserialize)]
        struct IssueCommentsData {
            issue: IssueWithComments,
        }
        
        #[derive(Debug, Deserialize)]
        struct IssueWithComments {
            comments: Connection<Comment>,
        }
        
        let data: IssueCommentsData = self.execute_query(&query, Some(variables)).await?;
        
        Ok(data.issue.comments.nodes)
    }

    pub async fn create_comment(&self, issue_id: &str, body: &str) -> Result<Comment, Box<dyn std::error::Error>> {
        let query = format!(
            r#"
            mutation($issueId: String!, $body: String!) {{
                commentCreate(input: {{ issueId: $issueId, body: $body }}) {{
                    success
                    comment {{
                        {}
                    }}
                }}
            }}
            "#,
            COMMENT_FIELDS
        );
        let variables = json!({ "issueId": issue_id, "body": body });
        let data: graphql::CommentCreateData = self.execute_query(&query, Some(variables)).await?;
        
        if data.comment_create.success {
            data.comment_create.comment.ok_or("Failed to create comment".into())
        } else {
            Err("Failed to create comment".into())
        }
    }

    pub async fn update_comment(&self, comment_id: &str, body: &str) -> Result<Comment, Box<dyn std::error::Error>> {
        let query = format!(
            r#"
            mutation($id: String!, $body: String!) {{
                commentUpdate(id: $id, input: {{ body: $body }}) {{
                    success
                    comment {{
                        {}
                    }}
                }}
            }}
            "#,
            COMMENT_FIELDS
        );
        let variables = json!({ "id": comment_id, "body": body });
        let data: graphql::CommentUpdateData = self.execute_query(&query, Some(variables)).await?;
        
        if data.comment_update.success {
            data.comment_update.comment.ok_or("Failed to update comment".into())
        } else {
            Err("Failed to update comment".into())
        }
    }

    pub async fn delete_comment(&self, comment_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let query = r#"
            mutation($id: String!) {
                commentDelete(id: $id) {
                    success
                }
            }
        "#;
        let variables = json!({ "id": comment_id });
        let data: graphql::CommentDeleteData = self.execute_query(query, Some(variables)).await?;
        
        Ok(data.comment_delete.success)
    }

    pub async fn update_issue_bulk(
        &self,
        issue_id: &str,
        state_id: Option<&str>,
        assignee_id: Option<&str>,
        priority: Option<u8>,
        add_label_ids: Option<&[String]>,
        remove_label_ids: Option<&[String]>,
    ) -> Result<Issue, Box<dyn std::error::Error>> {
        let mut input = json!({});
        
        if let Some(state_id) = state_id {
            input["stateId"] = json!(state_id);
        }
        if let Some(assignee_id) = assignee_id {
            input["assigneeId"] = json!(assignee_id);
        }
        if let Some(priority) = priority {
            input["priority"] = json!(priority);
        }
        if let Some(add_labels) = add_label_ids {
            input["labelIds"] = json!(add_labels);
        }
        if let Some(remove_labels) = remove_label_ids {
            // For removing labels, we need to get current labels and filter them
            // This is a simplified version - in production, you'd want to fetch current labels first
            input["removeLabelIds"] = json!(remove_labels);
        }
        
        let query = format!(
            r#"
            mutation($id: String!, $input: IssueUpdateInput!) {{
                issueUpdate(id: $id, input: $input) {{
                    success
                    issue {{
                        {}
                    }}
                }}
            }}
            "#,
            ISSUE_FIELDS
        );
        
        let variables = json!({
            "id": issue_id,
            "input": input
        });
        
        let data: graphql::IssueUpdateData = self.execute_query(&query, Some(variables)).await?;
        Self::check_success(data.issue_update.success, data.issue_update.issue, "Failed to update issue")
    }

    pub async fn get_workflow_states(&self) -> Result<Vec<WorkflowState>, Box<dyn std::error::Error>> {
        let query = r#"
            query {
                workflowStates(first: 50) {
                    nodes {
                        id
                        name
                        type
                        color
                        position
                    }
                }
            }
        "#;
        
        #[derive(Debug, Deserialize)]
        struct WorkflowStatesData {
            #[serde(rename = "workflowStates")]
            workflow_states: Connection<WorkflowState>,
        }
        
        let data: WorkflowStatesData = self.execute_query(query, None).await?;
        Ok(data.workflow_states.nodes)
    }

    pub async fn move_issue(
        &self,
        issue_id: &str,
        team_id: Option<&str>,
        project_id: Option<&str>,
    ) -> Result<Issue, Box<dyn std::error::Error>> {
        let mut input = json!({});
        
        if let Some(team_id) = team_id {
            input["teamId"] = json!(team_id);
        }
        if let Some(project_id) = project_id {
            input["projectId"] = json!(project_id);
        }
        
        let query = format!(
            r#"
            mutation($id: String!, $input: IssueUpdateInput!) {{
                issueUpdate(id: $id, input: $input) {{
                    success
                    issue {{
                        {}
                    }}
                }}
            }}
            "#,
            ISSUE_FIELDS
        );
        
        let variables = json!({
            "id": issue_id,
            "input": input
        });
        
        let data: graphql::IssueUpdateData = self.execute_query(&query, Some(variables)).await?;
        Self::check_success(data.issue_update.success, data.issue_update.issue, "Failed to move issue")
    }
}