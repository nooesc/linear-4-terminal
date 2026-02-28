#![allow(dead_code)]

use crate::error::{LinearError, LinearResult};
use crate::graphql_fields::FieldSelection;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Simplified GraphQL client with better error handling
pub struct GraphQLClient {
    client: Client,
    api_url: String,
    api_key: String,
}

impl GraphQLClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_url: "https://api.linear.app/graphql".to_string(),
            api_key,
        }
    }
    
    /// Execute a GraphQL query
    pub async fn query<T>(&self, query: &str) -> LinearResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.execute(query).await?;
        self.extract_data(response)
    }
    
    /// Execute a GraphQL mutation
    pub async fn mutate<T>(&self, mutation: &str) -> LinearResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.execute(mutation).await?;
        self.extract_data(response)
    }
    
    /// Execute a raw GraphQL request
    async fn execute(&self, query: &str) -> LinearResult<GraphQLResponse> {
        let request_body = GraphQLRequest {
            query: query.to_string(),
        };
        
        let response = self.client
            .post(&self.api_url)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| LinearError::RequestError(e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LinearError::ApiError(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }
        
        response
            .json::<GraphQLResponse>()
            .await
            .map_err(|e| LinearError::RequestError(e))
    }
    
    /// Extract data from GraphQL response, handling errors
    fn extract_data<T>(&self, response: GraphQLResponse) -> LinearResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        if let Some(errors) = response.errors {
            let error_messages = errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(LinearError::GraphQLError(error_messages));
        }
        
        match response.data {
            Some(data) => serde_json::from_value(data)
                .map_err(|e| LinearError::JsonError(e)),
            None => Err(LinearError::GraphQLError("No data in response".to_string())),
        }
    }
}

#[derive(Debug, Serialize)]
struct GraphQLRequest {
    query: String,
}

#[derive(Debug, Deserialize)]
struct GraphQLResponse {
    data: Option<Value>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQLError {
    message: String,
    #[serde(default)]
    extensions: Option<Value>,
}

/// Builder for GraphQL queries with field selection
pub struct QueryBuilder {
    operation: String,
    args: Vec<(String, String)>,
    selection: FieldSelection,
}

impl QueryBuilder {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            args: Vec::new(),
            selection: FieldSelection::new(),
        }
    }
    
    pub fn arg(mut self, name: &str, value: &str) -> Self {
        self.args.push((name.to_string(), value.to_string()));
        self
    }
    
    pub fn args(mut self, args: &[(&str, &str)]) -> Self {
        for (name, value) in args {
            self.args.push((name.to_string(), value.to_string()));
        }
        self
    }
    
    pub fn selection(mut self, selection: FieldSelection) -> Self {
        self.selection = selection;
        self
    }
    
    pub fn build(self) -> String {
        if self.args.is_empty() {
            format!("query {{ {} {{ {} }} }}", self.operation, self.selection)
        } else {
            let args_str = self.args
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("query {{ {}({}) {{ {} }} }}", self.operation, args_str, self.selection)
        }
    }
}

/// Builder for GraphQL mutations
pub struct MutationBuilder {
    operation: String,
    input: Option<Value>,
    args: Vec<(String, String)>,
    selection: FieldSelection,
}

impl MutationBuilder {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            input: None,
            args: Vec::new(),
            selection: FieldSelection::new(),
        }
    }
    
    pub fn input<T: Serialize>(mut self, input: T) -> Self {
        self.input = Some(serde_json::to_value(input).unwrap());
        self
    }
    
    pub fn arg(mut self, name: &str, value: &str) -> Self {
        self.args.push((name.to_string(), value.to_string()));
        self
    }
    
    pub fn selection(mut self, selection: FieldSelection) -> Self {
        self.selection = selection;
        self
    }
    
    pub fn build(self) -> String {
        let mut args = self.args;
        
        if let Some(input) = self.input {
            args.push(("input".to_string(), input.to_string()));
        }
        
        if args.is_empty() {
            format!("mutation {{ {} {{ {} }} }}", self.operation, self.selection)
        } else {
            let args_str = args
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            format!("mutation {{ {}({}) {{ {} }} }}", self.operation, args_str, self.selection)
        }
    }
}