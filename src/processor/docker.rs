use crate::consts::IMAGE_NAME;
use anyhow::Result;
use bollard::{
    container::{Config, CreateContainerOptions, PruneContainersOptions, WaitContainerOptions},
    image::{CreateImageOptions, ListImagesOptions},
    secret::HostConfig,
    Docker,
};
use futures::TryStreamExt;
use std::collections::HashMap;
use tokio_stream::StreamExt;

pub async fn prune_containers() -> Result<()> {
    let docker = Docker::connect_with_local_defaults().unwrap();

    docker
        .prune_containers(Some(PruneContainersOptions {
            filters: HashMap::<&str, Vec<&str>>::new(),
        }))
        .await?;

    Ok(())
}

pub async fn remove_container(id: String) -> Result<()> {
    let docker = Docker::connect_with_local_defaults()?;

    docker.remove_container(&id, None).await?;

    Ok(())
}

pub async fn build_program(
    id: &str,
    project_path: &str,
    repo_url: &str,
    project_name: Option<String>,
    build_idl: bool,
    version: &str,
) -> Result<String> {
    let docker = Docker::connect_with_local_defaults()?;

    let cc_options = CreateContainerOptions {
        name: id,
        platform: None,
    };

    let mount = format!("{}:/mnt/build", project_path);
    let mut volumes: HashMap<&str, HashMap<(), ()>> = HashMap::default();
    volumes.insert(&mount, HashMap::default());

    let repo_url_env = format!("REPO_URL={}", repo_url);
    let project_name_env = format!("PROJECT_NAME={}", project_name.unwrap_or("".into()));
    let mut env: Vec<&str> = vec![&repo_url_env, &project_name_env];
    if build_idl {
        env.push("BUILD_IDL=true");
    }

    let image = format!("{}:{}", IMAGE_NAME, version);

    let cc_config = Config {
        image: Some(image.as_str()),
        env: Some(env),
        host_config: Some(HostConfig {
            binds: Some(vec![mount.clone()]),
            ..Default::default()
        }),
        volumes: Some(volumes),
        ..Default::default()
    };

    let id = docker
        .create_container(Some(cc_options), cc_config)
        .await?
        .id;

    docker.start_container::<String>(&id, None).await?;

    docker
        .wait_container(
            &id,
            Some(WaitContainerOptions {
                condition: "not-running",
            }),
        )
        .try_collect::<Vec<_>>()
        .await?;

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
        let tag = tags[0].clone();
        if tag == format!("{}:{}", IMAGE_NAME, version) {
            return Ok(true);
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

    let options = CreateImageOptions {
        from_image: format!("{}:{}", IMAGE_NAME, version),
        ..Default::default()
    };

    let mut create_stream = docker.create_image(Some(options), None, None);

    while let Some(msg) = create_stream.next().await {
        if let Err(msg) = msg {
            log::error!("Failed to pull image {}. {msg:?}", version);
        }
    }

    Ok(())
}
