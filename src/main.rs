use dotenvy::dotenv;
use sails_program_verifier::{
    build_verifier_image,
    consts::AVAILABLE_VERSIONS,
    db::{get_connection_pool, Verification},
    prune_containers, remove_dangling_images, run_processor, run_server,
    util::{clean_or_create_logs_dir, create_verifier_dockerfile},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Prunning old containers");
    prune_containers().await?;

    for v in AVAILABLE_VERSIONS {
        log::info!("Creating verifier dockerfile and image for version {v}");
        create_verifier_dockerfile(v)?;
        build_verifier_image(v).await?;
    }

    log::info!("Removing dangling images");
    remove_dangling_images().await?;

    log::info!("Cleaning logs directory");
    clean_or_create_logs_dir()?;

    log::info!("Connecting to the database");
    let pool = Arc::new(get_connection_pool());

    let server_pool = Arc::clone(&pool);
    let proc_pool = Arc::clone(&pool);

    log::info!("Resetting in progress verifications");
    Verification::reset_in_progress(&mut server_pool.get().unwrap())?;

    let proc_handle = run_processor(proc_pool).await?;

    tokio::spawn(async move {
        proc_handle.await.unwrap();
        log::info!("Builder started successfully");
    });

    tokio::spawn(async move {
        run_server(server_pool).await;
        log::info!("Server started successfully");
    })
    .await?;

    Ok(())
}
