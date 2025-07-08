use super::docker::{build_program, remove_container};
use crate::{consts::PATH_TO_BUILDS, db::Verification, util::generate_code_id};
use anyhow::{bail, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct BuildArtifacts {
    pub code_id: String,
    pub idl: Option<String>,
    pub name: String,
}

fn get_project_path(id: &str) -> PathBuf {
    Path::new(PATH_TO_BUILDS).join(id)
}

pub async fn build_project(verif: Verification) -> Result<BuildArtifacts> {
    let proj_path = get_project_path(&verif.id);

    fs::create_dir_all(&proj_path)?;
    log::info!("{}: project dir created ({:?})", &verif.id, &proj_path);

    build_program(&verif, proj_path.to_str().unwrap()).await?;
    log::info!("{}: program built", &verif.id);

    let built_files = fs::read_dir(&proj_path)?;

    let mut wasm_path: Option<PathBuf> = None;
    let mut idl_path: Option<PathBuf> = None;

    for entry in built_files {
        let path = entry.as_ref().unwrap().path().to_str().unwrap().to_string();
        log::debug!("{:?} file found", &path);
        if path.ends_with(".opt.wasm") {
            wasm_path = Some(entry.as_ref().unwrap().path());
        } else if path.ends_with(".idl") {
            idl_path = Some(entry.as_ref().unwrap().path());
        }
    }

    let Some(wasm_path) = wasm_path else {
        bail!("Failed to build wasm.");
    };

    log::info!("{}: wasm - {:?}", &verif.id, &wasm_path);

    let code = fs::read(&wasm_path)?;
    let code_id = generate_code_id(&code);
    let code_filename = wasm_path.file_name().unwrap().to_str().unwrap();

    let idl = if verif.build_idl {
        let Some(idl_path) = idl_path else {
            bail!("Failed to build idl file.");
        };
        log::info!("{}: idl - {:?}", &verif.id, &idl_path);
        fs::read_to_string(&idl_path).ok()
    } else {
        None
    };

    Ok(BuildArtifacts {
        code_id,
        idl,
        name: code_filename[..(code_filename.len() - ".opt.wasm".len())].to_string(),
    })
}

pub async fn cleanup(verif_id: &str) -> Result<()> {
    // fs::remove_dir_all(get_project_path(verif_id))?;
    log::info!("{verif_id}: project dir cleaned");

    // remove_container(verif_id).await?;

    Ok(())
}
