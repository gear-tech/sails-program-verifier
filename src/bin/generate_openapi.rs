use sails_program_verifier::ApiDoc;
use std::fs;
use utoipa::OpenApi;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let openapi = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi)?;

    fs::write("openapi.json", json)?;
    println!("OpenAPI specification generated: openapi.json");

    Ok(())
}
