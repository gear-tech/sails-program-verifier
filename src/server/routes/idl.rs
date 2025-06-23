use crate::{common::Pool, db::Idl, server::types::IdQueryParams, util::validate_and_get_code_id};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

#[utoipa::path(get, path="/idl", params(IdQueryParams), responses(
    (status = 200, description="Idl by id", body=Idl)
))]
pub async fn idl(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<IdQueryParams>,
) -> Result<Json<Idl>, StatusCode> {
    let conn = &mut pool.get().unwrap();

    let Ok(code_id) = validate_and_get_code_id(&params.id) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    if let Some(idl) = Idl::get(conn, &code_id) {
        Ok(Json(idl))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
