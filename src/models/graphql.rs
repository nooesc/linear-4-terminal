use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Deserialize)]
pub struct GraphQLError {
    pub message: String,
}

// Viewer data structures
#[derive(Debug, Deserialize)]
pub struct ViewerData {
    pub viewer: super::User,
}

// Issue data structures
#[derive(Debug, Deserialize)]
pub struct IssuesData {
    pub issues: super::Connection<super::Issue>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct IssueWithComments {
    pub issue: super::Issue,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CommentsData {
    pub comments: super::Connection<super::Comment>,
}

// Team data structures
#[derive(Debug, Deserialize)]
pub struct TeamsData {
    pub teams: super::Connection<super::Team>,
}

// Project data structures
#[derive(Debug, Deserialize)]
pub struct ProjectsData {
    pub projects: super::Connection<super::Project>,
}

// Mutation response structures
#[derive(Debug, Deserialize)]
pub struct IssueMutationPayload {
    pub success: bool,
    pub issue: Option<super::Issue>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectMutationPayload {
    pub success: bool,
    pub project: Option<super::Project>,
}

#[derive(Debug, Deserialize)]
pub struct ArchivePayload {
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct CommentMutationPayload {
    pub success: bool,
    pub comment: Option<super::Comment>,
}

// Create mutation data structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCreateData {
    pub issue_create: IssueMutationPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectCreateData {
    pub project_create: ProjectMutationPayload,
}

// Update mutation data structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueUpdateData {
    pub issue_update: IssueMutationPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdateData {
    pub project_update: ProjectMutationPayload,
}

// Archive mutation data structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueArchiveData {
    pub issue_archive: ArchivePayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArchiveData {
    pub project_archive: ArchivePayload,
}

// Comment mutation data structures
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentCreateData {
    pub comment_create: CommentMutationPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentUpdateData {
    pub comment_update: CommentMutationPayload,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentDeleteData {
    pub comment_delete: ArchivePayload,
}