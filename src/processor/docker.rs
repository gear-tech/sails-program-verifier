use crate::{consts::IMAGE_NAME, db::Verification};
use anyhow::{bail, Result};
use bollard::{
    auth::DockerCredentials,
    body_full,
    models::ContainerCreateBody,
    query_parameters::{
        BuildImageOptions, CreateContainerOptions, CreateImageOptions, ListContainersOptions,
        ListImagesOptions, LogsOptions, RemoveContainerOptions, RemoveImageOptions,
        StartContainerOptions, WaitContainerOptions,
    },
    secret::{HostConfig, Mount, MountTypeEnum},
    Docker,
};
use futures::{StreamExt, TryStreamExt};
use std::env;
use tar::Builder;

pub async fn prune_containers() -> Result<()> {
    let docker = Docker::connect_with_local_defaults().unwrap();

    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters: None,
            ..Default::default()
        }))
        .await?;

    for c in containers {
        let id = c.id.unwrap();
        docker
            .remove_container(
                &id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
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

    let cc_options = CreateContainerOptions {
        name: Some(verif.id.to_string()),
        platform: "".to_string(),
    };

    let repo_url_env = format!("REPO_URL={}", &verif.repo_link);
    let project_name_env = format!(
        "PROJECT_NAME={}",
        verif.project_name.clone().unwrap_or_default()
    );
    let manifest_path_env = format!(
        "MANIFEST_PATH={}",
        &verif.manifest_path.clone().unwrap_or_default()
    );

    let mut env: Vec<String> = vec![repo_url_env, project_name_env, manifest_path_env];

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
            Some(WaitContainerOptions {
                condition: "not-running".to_string(),
            }),
        )
        .try_collect::<Vec<_>>()
        .await?;

    let logs = docker.logs(
        &id,
        Some(LogsOptions {
            stdout: true,
            stderr: true,
            ..Default::default()
        }),
    );

    logs.for_each(|l| async move {
        log::debug!("{}: {:?}", &verif.id, l);
    })
    .await;

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

async fn does_image_exist(image_name: &str, docker: &Docker) -> Result<bool> {
    let images = docker
        .list_images(Some(ListImagesOptions {
            all: true,
            filters: None,
            ..Default::default()
        }))
        .await?;

    for image in images {
        let tags = image.repo_tags;
        if tags.is_empty() {
            continue;
        }
        for tag in tags {
            if tag == image_name {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

pub async fn pull_docker_image(version: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;

    if does_image_exist(version, &docker).await? {
        return Ok(());
    }

    log::info!("Pulling image w/ version {}", version);

    let auth_config = DockerCredentials {
        username: env::var("DOCKER_USERNAME").ok(),
        password: env::var("DOCKER_ACCESS_TOKEN").ok(),
        serveraddress: Some("ghcr.io".to_string()),
        ..Default::default()
    };

    let options = CreateImageOptions {
        from_image: Some(format!("{}:{}", IMAGE_NAME, version)),
        ..Default::default()
    };

    let mut create_stream = docker.create_image(Some(options), None, Some(auth_config));

    while let Some(msg) = create_stream.next().await {
        if let Err(msg) = msg {
            log::error!("Failed to pull image {version}. {msg:?}");
        }
    }

    Ok(())
}

pub async fn build_verifier_image(version: &str) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;
    let image_name = format!("verifier:{version}");

    if does_image_exist(version, &docker).await? {
        docker
            .remove_image(
                &image_name,
                Some(RemoveImageOptions {
                    force: true,
                    ..Default::default()
                }),
                None,
            )
            .await?;
    }

    let options = BuildImageOptions {
        dockerfile: "Dockerfile".to_string(),
        t: Some(image_name),
        ..Default::default()
    };

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
