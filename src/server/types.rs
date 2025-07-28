use crate::db::Code;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Default, Deserialize, Debug, ToSchema)]
pub enum Project {
    #[default]
    Root,
    /// Name of the package to be built and its base path (optional)
    Package(String),
    /// Manifest path of the package
    ManifestPath(String),
}

#[derive(Serialize, ToSchema)]
pub struct StatusResponse {
    /// Status of the verification
    pub status: String,
    /// Reason for failure, if any
    pub failed_reason: Option<String>,
    /// Timestamp of the verification
    pub created_at: u128,
}

#[derive(Deserialize, IntoParams)]
pub struct IdQueryParams {
    /// ID
    pub id: String,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct VerifyRequest {
    /// Link to the repository containing the code to be verified.
    pub repo_link: String,
    /// Version of the Docker image to use for verification.
    pub version: String,
    /// Project to verify (default: root)
    pub project: Option<Project>,
    /// Base path of the package to be built (default: root)
    pub base_path: Option<String>,
    /// Network where the code of the program is deployed
    pub network: String,
    /// ID of the deployed code
    pub code_id: String,
    /// Whether to build the IDL (default: false)
    pub build_idl: Option<bool>,
}

#[derive(serde::Serialize, ToSchema)]
pub struct VerifyResponse {
    /// ID of the verification
    pub id: String,
}

#[derive(Deserialize, IntoParams)]
pub struct CodeIdsQueryParams {
    /// List of code ids
    pub ids: Vec<String>,
}

#[derive(Serialize, ToSchema)]
pub struct CodesResponseEntry {
    /// Code id
    pub id: String,
    /// Code details
    pub code: Option<Code>,
}
