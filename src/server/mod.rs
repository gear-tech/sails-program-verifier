use crate::{
    common::Pool,
    consts::AVAILABLE_VERSIONS,
    db::{Code, Idl, Verification, VerificationStatus},
    util::{check_docker_version, generate_id},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use error::AppError;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::UNIX_EPOCH};
use utoipa::{IntoParams, OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

mod error;

#[derive(Default, Deserialize, Debug, ToSchema)]
enum Project {
    #[default]
    Root,
    Name(String),
    PathToCargoToml(String),
}

#[derive(Deserialize, Debug, ToSchema)]
struct VerifyRequest {
    pub repo_link: String,
    pub version: String,
    pub project: Option<Project>,
    pub network: String,
    pub code_id: String,
    pub build_idl: Option<bool>,
}

#[derive(serde::Serialize, ToSchema)]
struct VerifyResponse {
    pub id: String,
}

#[utoipa::path(post, path="/verify", request_body=VerifyRequest, responses(
    (status = 200, description="Verification request accepted", body=VerifyResponse)
))]
async fn verify(
    State(pool): State<Arc<Pool>>,
    Json(VerifyRequest {
        repo_link,
        code_id,
        project,
        version,
        network,
        build_idl,
    }): Json<VerifyRequest>,
) -> Result<Json<VerifyResponse>, AppError> {
    let verification_id = generate_id();

    check_docker_version(&version)?;

    let (project_name, path_to_cargo_toml) = match project.unwrap_or_default() {
        Project::Root => (None, None),
        Project::Name(name) => (Some(name), None),
        Project::PathToCargoToml(path) => (None, Some(path)),
    };

    Verification::save(
        &mut pool.get().unwrap(),
        Verification {
            id: verification_id.clone(),
            repo_link,
            code_id,
            project_name,
            path_to_cargo_toml,
            version,
            status: VerificationStatus::Pending,
            network: network.try_into()?,
            build_idl: build_idl.unwrap_or(true),
            failed_reason: None,
            created_at: std::time::SystemTime::now(),
        },
    );

    Ok(Json(VerifyResponse {
        id: verification_id,
    }))
}

#[derive(Serialize, ToSchema)]
struct StatusResponse {
    pub status: String,
    pub failed_reason: Option<String>,
    pub created_at: u128,
}

#[derive(Deserialize, IntoParams)]
struct IdQueryParams {
    id: String,
}

#[utoipa::path(get, path="/verify/status", params(IdQueryParams), responses(
    (status = 200, description="Status of the verification request", body=StatusResponse)
))]
async fn status(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<IdQueryParams>,
) -> Result<Json<StatusResponse>, StatusCode> {
    let conn = &mut pool.get().unwrap();

    if let Some(verif) = Verification::get(conn, &params.id) {
        Ok(Json(StatusResponse {
            status: verif.status.into(),
            failed_reason: verif.failed_reason,
            created_at: verif
                .created_at
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(get, path="/code", params(IdQueryParams), responses(
    (status = 200, description="Code by id", body=Code)
))]
async fn code(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<IdQueryParams>,
) -> Result<Json<Code>, StatusCode> {
    let conn = &mut pool.get().unwrap();

    if let Some(code) = Code::get(conn, &params.id) {
        Ok(Json(code))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(get, path="/idl", params(IdQueryParams), responses(
    (status = 200, description="Idl by id", body=Idl)
))]
async fn idl(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<IdQueryParams>,
) -> Result<Json<Idl>, StatusCode> {
    let conn = &mut pool.get().unwrap();

    if let Some(idl) = Idl::get(conn, &params.id) {
        Ok(Json(idl))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[utoipa::path(get, path="/version", responses(
    (status = 200, description="Version of the server", body=String)
))]
async fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[utoipa::path(get, path="/supported_versions", responses(
    (status = 200, description="Supported Sails versions", body=Vec<String>)
))]
async fn supported_versions() -> Json<Vec<&'static str>> {
    Json(AVAILABLE_VERSIONS.to_vec())
}

#[derive(OpenApi)]
#[openapi(
    paths(verify, status, code, idl, supported_versions),
    components(schemas(VerifyRequest, VerifyResponse, Code, Idl, StatusResponse))
)]
struct ApiDoc;

pub async fn run_server(pool: Arc<Pool>) {
    let app = Router::new()
        .route("/verify", post(verify))
        .route("/verify/status", get(status))
        .route("/code", get(code))
        .route("/idl", get(idl))
        .route("/version", get(version))
        .route("/supported_versions", get(supported_versions))
        .with_state(pool)
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    log::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
