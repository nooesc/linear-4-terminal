use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Issue {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub priority: Option<u8>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    pub state: WorkflowState,
    pub assignee: Option<super::User>,
    pub team: super::Team,
    pub labels: LabelConnection,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkflowState {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub state_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LabelConnection {
    pub nodes: Vec<Label>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub color: String,
}