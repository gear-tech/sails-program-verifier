use crate::{consts::IMAGE_NAME, db::Verification};
use anyhow::Result;
use bollard::{
    auth::DockerCredentials,
    container::{
        Config, CreateContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions,
        WaitContainerOptions,
    },
    image::{CreateImageOptions, ListImagesOptions},
    secret::HostConfig,
    Docker,
};
use futures::{StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::env;

pub async fn prune_containers() -> Result<()> {
    let docker = Docker::connect_with_local_defaults().unwrap();

    let filters: HashMap<String, Vec<String>> = HashMap::new();

    let containers = docker
        .list_containers(Some(ListContainersOptions {
            all: true,
            filters,
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

    docker.remove_container(id, None).await?;

    log::info!("{id}: container removed");

    Ok(())
}

pub async fn build_program(verif: &Verification, project_path: &str) -> Result<String> {
    log::debug!("{}: Start building program. {}", &verif.id, project_path);
    let docker = Docker::connect_with_local_defaults()?;

    let cc_options = CreateContainerOptions {
        name: &verif.id,
        platform: None,
    };

    let mount = format!("{}:/mnt/target", project_path);
    let mut volumes: HashMap<&str, HashMap<(), ()>> = HashMap::default();
    volumes.insert(&mount, HashMap::default());

    let repo_url_env = format!("REPO_URL={}", &verif.repo_link);
    let project_name_env = format!(
        "PROJECT_NAME={}",
        verif.project_name.clone().unwrap_or_default()
    );
    let manifest_path_env = format!(
        "MANIFEST_PATH={}",
        &verif.manifest_path.clone().unwrap_or_default()
    );

    let mut env: Vec<&str> = vec![&repo_url_env, &project_name_env, &manifest_path_env];

    if verif.build_idl {
        env.push("BUILD_IDL=true");
    }

    let image = format!("{}:{}", IMAGE_NAME, &verif.version);

    let cc_config = Config {
        image: Some(image.as_str()),
        env: Some(env),
        host_config: Some(HostConfig {
            binds: Some(vec![mount.clone()]),
            ..Default::default()
        }),
        volumes: Some(volumes),
        attach_stderr: Some(true),
        attach_stdout: Some(true),
        ..Default::default()
    };

    let id = docker
        .create_container(Some(cc_options), cc_config)
        .await?
        .id;

    log::info!("{}: container created({})", &verif.id, &id[0..12]);

    docker.start_container::<String>(&id, None).await?;

    log::info!("{}: container started({})", &verif.id, &id[0..12]);

    let c_result = docker
        .wait_container(
            &id,
            Some(WaitContainerOptions {
                condition: "not-running",
            }),
        )
        .try_collect::<Vec<_>>()
        .await?;

    let logs = docker.logs(
        &id,
        Some(LogsOptions::<String> {
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

async fn does_image_exist(version: &str, docker: &Docker) -> Result<bool> {
    let images = docker
        .list_images(Some(ListImagesOptions {
            all: true,
            filters: HashMap::<&str, Vec<&str>>::new(),
            ..Default::default()
        }))
        .await?;

    for image in images {
        let tags = image.repo_tags;
        if tags.is_empty() {
            continue;
        }
        for tag in tags {
            if tag == format!("{}:{}", IMAGE_NAME, version) {
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
        from_image: format!("{}:{}", IMAGE_NAME, version),
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
