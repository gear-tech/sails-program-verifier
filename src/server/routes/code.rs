use crate::server::types::{CodeIdsQueryParams, CodesResponseEntry, IdQueryParams};
use crate::{common::Pool, db::Code, util::validate_and_get_code_id};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[utoipa::path(get, path="/codes", params(CodeIdsQueryParams), responses(
    (status = 200, description="Get list of codes by its ids", body=Vec<CodesResponseEntry>)
))]
pub async fn codes(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<CodeIdsQueryParams>,
) -> Result<Json<Vec<CodesResponseEntry>>, StatusCode> {
    let conn = &mut pool.get().unwrap();

    let mut result: Vec<CodesResponseEntry> = Vec::new();
    let mut id_map = HashMap::new();
    let validated_ids: Vec<String> = params
        .ids
        .iter()
        .filter_map(|id| match validate_and_get_code_id(id) {
            Ok(code_id) => {
                id_map.insert(code_id.clone(), id.clone());
                Some(code_id)
            }
            Err(_) => {
                result.push(CodesResponseEntry {
                    id: id.clone(),
                    code: None,
                });
                None
            }
        })
        .collect();

    match Code::get_many(conn, &validated_ids) {
        Ok(codes) => {
            let mut found_ids = HashSet::new();
            for code in codes {
                found_ids.insert(code.id.clone());
                result.push(CodesResponseEntry {
                    id: id_map.get(&code.id).unwrap().into(),
                    code: Some(code),
                });
            }
            for id in &validated_ids {
                if !found_ids.contains(id) {
                    if let Some(id) = id_map.get(id) {
                        result.push(CodesResponseEntry {
                            id: id.into(),
                            code: None,
                        });
                    }
                }
            }
            Ok(Json(result))
        }
        Err(error) => {
            log::error!("Failed to get codes from db {error:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(get, path="/code", params(IdQueryParams), responses(
    (status = 200, description="Code by id", body=Code)
))]
pub async fn code(
    State(pool): State<Arc<Pool>>,
    Query(params): Query<IdQueryParams>,
) -> Result<Json<Code>, StatusCode> {
    let conn = &mut pool.get().unwrap();

    let Ok(code_id) = validate_and_get_code_id(&params.id) else {
        return Err(StatusCode::BAD_REQUEST);
    };

    if let Some(code) = Code::get(conn, &code_id) {
        Ok(Json(code))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
