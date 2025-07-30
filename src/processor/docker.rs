use crate::{
    consts::{IMAGE_NAME, LOGS_DIR},
    db::Verification,
};
use anyhow::{bail, Result};
use bollard::{
    body_full,
    models::ContainerCreateBody,
    query_parameters::{
        BuildImageOptionsBuilder, CreateContainerOptionsBuilder, ListContainersOptionsBuilder,
        LogsOptionsBuilder, PruneImagesOptionsBuilder, RemoveContainerOptions,
        RemoveContainerOptionsBuilder, StartContainerOptions, WaitContainerOptionsBuilder,
    },
    secret::{HostConfig, Mount, MountTypeEnum},
    Docker,
};
use futures::{StreamExt, TryStreamExt};
use std::{collections::HashMap, io::Write};
use tar::Builder;

pub async fn prune_containers() -> Result<()> {
    let docker = Docker::connect_with_local_defaults().unwrap();

    let containers = docker
        .list_containers(Some(ListContainersOptionsBuilder::new().all(true).build()))
        .await?;

    for c in containers {
        let id = c.id.unwrap();
        docker
            .remove_container(
                &id,
                Some(RemoveContainerOptionsBuilder::new().force(true).build()),
            )
            .await?;
    }

    Ok(())
}

pub async fn remove_container(id: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;

    docker
        .remove_container(id, None::<RemoveContainerOptions>)
        .await?;

    log::info!("{id}: container removed");

    Ok(())
}

pub async fn build_program(verif: &Verification, project_path: &str) -> Result<String> {
    log::debug!("{}: Start building program. {}", &verif.id, project_path);
    let docker = Docker::connect_with_local_defaults()?;

    let cc_options = CreateContainerOptionsBuilder::new().name(&verif.id).build();

    let repo_url_env = format!("REPO_URL={}", &verif.repo_link);
    let project_name_env = format!(
        "PROJECT_NAME={}",
        verif.project_name.clone().unwrap_or_default()
    );
    let manifest_path_env = format!(
        "MANIFEST_PATH={}",
        &verif.manifest_path.clone().unwrap_or_default()
    );
    let base_path_env = format!("BASE_PATH={}", &verif.base_path.clone().unwrap_or_default());

    let mut env: Vec<String> = vec![
        repo_url_env,
        project_name_env,
        manifest_path_env,
        base_path_env,
    ];

    if verif.build_idl {
        env.push("BUILD_IDL=true".to_string());
    }

    let image = format!("{}:{}", IMAGE_NAME, &verif.version);

    let mount = Mount {
        source: Some(project_path.to_string()),
        target: Some("/mnt/target".to_string()),
        read_only: Some(false),
        typ: Some(MountTypeEnum::BIND),
        ..Default::default()
    };

    let cc_config = ContainerCreateBody {
        image: Some(image),
        env: Some(env),
        host_config: Some(HostConfig {
            mounts: Some(vec![mount]),
            ..Default::default()
        }),
        attach_stderr: Some(true),
        attach_stdout: Some(true),
        ..Default::default()
    };

    let id = docker
        .create_container(Some(cc_options), cc_config)
        .await?
        .id;

    log::info!("{}: container created({})", &verif.id, &id[0..12]);

    docker
        .start_container(&id, Some(StartContainerOptions::default()))
        .await?;

    log::info!("{}: container started({})", &verif.id, &id[0..12]);

    let c_result = docker
        .wait_container(
            &id,
            Some(
                WaitContainerOptionsBuilder::new()
                    .condition("not-running")
                    .build(),
            ),
        )
        .try_collect::<Vec<_>>()
        .await?;

    let mut logs = docker.logs(
        &id,
        Some(LogsOptionsBuilder::new().stdout(true).stderr(true).build()),
    );

    let log_file_path = format!("{}/{}.log", LOGS_DIR, &verif.id);
    let mut log_file = std::fs::File::create(&log_file_path)?;

    while let Some(log_chunk) = logs.next().await {
        match log_chunk {
            Ok(chunk) => {
                log_file.write_all(&chunk.into_bytes())?;
            }
            Err(e) => {
                log::error!("{}: Failed to read log chunk: {:?}", &verif.id, e);
            }
        }
    }
    log_file.flush()?;

    for r in c_result {
        log::info!(
            "{}: error: {:?} || status code: {}",
            &verif.id,
            r.error,
            r.status_code
        );
    }

    log::info!("{}: container exited({})", &verif.id, &id[0..12]);

    Ok(id)
}

pub async fn build_verifier_image(version: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    let image_name = format!("verifier:{version}");

    let options = BuildImageOptionsBuilder::new()
        .dockerfile("Dockerfile")
        .t(&image_name)
        .build();

    let mut tar_content = Vec::new();

    {
        let mut tar_builder = Builder::new(&mut tar_content);
        tar_builder
            .append_path_with_name(format!("Dockerfile-verifier-{version}"), "Dockerfile")?;
        tar_builder.append_path("build.sh")?;
        tar_builder.finish()?;
    }

    let mut build_stream = docker.build_image(options, None, Some(body_full(tar_content.into())));

    while let Some(msg) = build_stream.next().await {
        if let Err(msg) = msg {
            bail!("Failed to build image {version}. {msg:?}")
        }
    }

    Ok(())
}

pub async fn remove_dangling_images() -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;

    let filters = HashMap::from([("dangling", vec!["true"])]);

    docker
        .prune_images(Some(
            PruneImagesOptionsBuilder::new().filters(&filters).build(),
        ))
        .await?;

    Ok(())
}
