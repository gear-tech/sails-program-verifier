use super::docker::{build_program, remove_container};
use crate::{consts::PATH_TO_BUILDS, db::Verification, util::generate_code_id};
use anyhow::bail;
use std::{fs, path::Path};

pub struct BuildArtifacts {
    pub code_id: String,
    pub idl: Option<String>,
    pub name: String,
}

pub async fn build_code(verif: Verification) -> anyhow::Result<BuildArtifacts> {
    let project_path = Path::new(PATH_TO_BUILDS).join(&verif.id);

    fs::create_dir_all(&project_path)?;

    let c_id = build_program(
        &verif.id,
        project_path.to_str().unwrap(),
        &verif.repo_link,
        verif.project_name.clone(),
        verif.build_idl,
        verif.version.as_str(),
    )
    .await?;

    remove_container(c_id).await?;

    let built_files = fs::read_dir(&project_path)?;

    let mut wasm_path: Option<String> = None;
    let mut idl_path: Option<String> = None;

    for entry in built_files {
        let path = entry.as_ref().unwrap().path().to_str().unwrap().to_string();
        if path.ends_with(".opt.wasm") {
            wasm_path = Some(path);
        } else if path.ends_with(".idl") {
            idl_path = Some(path);
        }
    }

    let Some(wasm_path) = wasm_path else {
        bail!("Failed to build wasm.");
    };

    let code = fs::read(&wasm_path)?;
    let code_id = generate_code_id(&code);

    let idl = if verif.build_idl {
        let Some(idl_path) = idl_path else {
            bail!("Failed to build idl file.");
        };
        fs::read_to_string(&idl_path).ok()
    } else {
        None
    };

    fs::remove_dir_all(&project_path)?;

    Ok(BuildArtifacts {
        code_id,
        idl,
        name: wasm_path[..(wasm_path.len() - ".opt.wasm".len())].to_string(),
    })
}
