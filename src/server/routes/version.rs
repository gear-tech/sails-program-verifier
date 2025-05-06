use crate::consts::AVAILABLE_VERSIONS;
use axum::Json;

#[utoipa::path(get, path="/version", responses(
    (status = 200, description="Version of the server", body=String)
))]
pub async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[utoipa::path(get, path="/supported_versions", responses(
    (status = 200, description="Supported Sails versions", body=Vec<String>)
))]
pub async fn supported_versions() -> Json<Vec<&'static str>> {
    Json(AVAILABLE_VERSIONS.to_vec())
}
