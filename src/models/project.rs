use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub state: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}