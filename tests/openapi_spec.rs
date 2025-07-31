use insta::assert_snapshot;
use sails_program_verifier::ApiDoc;
use utoipa::OpenApi;

#[test]
fn test_openapi_specification() {
    let openapi = ApiDoc::openapi();

    let json = serde_json::to_string_pretty(&openapi).unwrap();

    assert_snapshot!(json);
}
