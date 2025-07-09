use crate::server::error::AppError;
use crate::server::types::{IdQueryParams, Project, StatusResponse, VerifyRequest, VerifyResponse};
use crate::{
    common::Pool,
    db::{Verification, VerificationStatus},
    util::{check_docker_version, generate_id, validate_and_get_code_id},
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use std::{sync::Arc, time::UNIX_EPOCH};

#[utoipa::path(post, path="/verify", request_body=VerifyRequest, responses(
    (status = 200, description="Verification request accepted", body=VerifyResponse)
))]
pub async fn verify(
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

    let (project_name, manifest_path) = match project.unwrap_or_default() {
        Project::Root => (None, None),
        Project::Name(name) => (Some(name), None),
        Project::ManifestPath(path) => (None, Some(path)),
    };

    let code_id = validate_and_get_code_id(&code_id)?;

    Verification::save(
        &mut pool.get().unwrap(),
        Verification {
            id: verification_id.clone(),
            repo_link,
            code_id,
            project_name,
            manifest_path,
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

#[utoipa::path(get, path="/verify/status", params(IdQueryParams), responses(
    (status = 200, description="Status of the verification request", body=StatusResponse)
))]
pub async fn status(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<IdQueryParams>,
) -> Result<Json<StatusResponse>, StatusCode> {
    let time_start = std::time::SystemTime::now();
    let conn = &mut pool.get().unwrap();

    if let Some(verif) = Verification::get(conn, &params.id) {
        let result = Ok(Json(StatusResponse {
            status: verif.status.into(),
            failed_reason: verif.failed_reason,
            created_at: verif
                .created_at
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis(),
        }));
        let time_end = std::time::SystemTime::now();
        let duration = time_end
            .duration_since(time_start)
            .expect("Time went backwards");
        log::debug!(
            "Verification status request took {}ms",
            duration.as_millis()
        );
        result
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
