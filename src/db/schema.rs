// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "network"))]
    pub struct Network;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "verificationstatus"))]
    pub struct Verificationstatus;
}

diesel::table! {
    code (id) {
        id -> Varchar,
        idl_hash -> Nullable<Varchar>,
        name -> Varchar,
        repo_link -> Varchar,
    }
}

diesel::table! {
    idl (id) {
        id -> Varchar,
        content -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Verificationstatus;
    use super::sql_types::Network;

    verification (id) {
        id -> Varchar,
        repo_link -> Varchar,
        code_id -> Varchar,
        project_name -> Nullable<Varchar>,
        build_idl -> Bool,
        version -> Varchar,
        status -> Verificationstatus,
        network -> Network,
        failed_reason -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(code, idl, verification,);
