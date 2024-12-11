use program_verifier::{
    consts::AVAILABLE_VERSIONS,
    db::{get_connection_pool, Verification},
    prune_containers, pull_docker_image, run_processor, run_server,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Prunning old containers");
    prune_containers().await?;

    for v in AVAILABLE_VERSIONS {
        pull_docker_image(v).await?;
    }

    let pool = Arc::new(get_connection_pool());

    log::info!("Connected to database");

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
