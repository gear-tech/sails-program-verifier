use crate::{common::Pool, db};
use axum::{
    routing::{get, post},
    Router,
};
use routes::{code, idl, verify, version};
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod error;
mod routes;
pub mod types;

#[derive(OpenApi)]
#[openapi(
    paths(
        verify::verify,
        verify::status,
        code::code,
        code::codes,
        idl::idl,
        version::supported_versions,
        version::version
    ),
    components(schemas(
        types::VerifyRequest,
        types::VerifyResponse,
        db::Code,
        db::Idl,
        types::StatusResponse
    ))
)]
pub struct ApiDoc;

pub async fn run_server(pool: Arc<Pool>) {
    let app = Router::new()
        .route("/verify", post(routes::verify::verify))
        .route("/verify/status", get(routes::verify::status))
        .route("/code", get(routes::code::code))
        .route("/codes", get(routes::code::codes))
        .route("/idl", get(routes::idl::idl))
        .route("/version", get(routes::version::version))
        .route(
            "/supported_versions",
            get(routes::version::supported_versions),
        )
        .with_state(pool)
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    log::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
