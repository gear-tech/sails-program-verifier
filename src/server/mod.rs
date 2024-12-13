use crate::{
    common::Pool,
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
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

mod error;

#[derive(Default, Deserialize, Debug)]
enum Project {
    #[default]
    Root,
    Name(String),
    PathToCargoToml(String),
}

#[derive(Deserialize, Debug)]
struct VerifyRequest {
    pub repo_link: String,
    pub version: String,
    pub project: Option<Project>,
    pub network: String,
    pub code_id: String,
    pub build_idl: Option<bool>,
}

#[derive(serde::Serialize)]
struct VerifyResponse {
    pub id: String,
}

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

#[derive(serde::Serialize)]
struct StatusResponse {
    pub status: String,
}

async fn status(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<StatusResponse>, StatusCode> {
    if let Some(id) = params.get("id") {
        let conn = &mut pool.get().unwrap();

        if let Some(verif) = Verification::get(conn, id) {
            Ok(Json(StatusResponse {
                status: verif.status.into(),
            }))
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn code(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Code>, StatusCode> {
    if let Some(id) = params.get("id") {
        let conn = &mut pool.get().unwrap();

        if let Some(code) = Code::get(conn, id) {
            Ok(Json(code))
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

async fn idl(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Idl>, StatusCode> {
    if let Some(id) = params.get("id") {
        let conn = &mut pool.get().unwrap();

        if let Some(idl) = Idl::get(conn, id) {
            Ok(Json(idl))
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else {
        Err(StatusCode::BAD_REQUEST)
    }
}

pub async fn run_server(pool: Arc<Pool>) {
    let app = Router::new()
        .route("/verify", post(verify))
        .route("/verify/status", get(status))
        .route("/code", get(code))
        .route("/idl", get(idl))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    log::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
