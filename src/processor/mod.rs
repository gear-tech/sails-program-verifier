use crate::{
    common::Pool,
    db::{Code, Idl, Network, Verification, VerificationStatus},
    util::hash_idl,
    Client,
};
use anyhow::{anyhow, bail, Result};
use builder::{build_project, cleanup};
use futures::{Stream, StreamExt};
use network_client::AppClients;
use std::{
    env,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{task::JoinHandle, time};
use tokio_stream::wrappers::IntervalStream;

mod builder;
mod docker;
pub mod network_client;
pub use docker::{build_verifier_image, prune_containers, pull_docker_image};

const MAX_VERIFS_IN_PROGRESS: i64 = 10;
const CHECK_INTERVAL: Duration = Duration::from_secs(30);

fn new_verifications(
    pool: Arc<Pool>,
    verifs_in_progress: Arc<AtomicI64>,
) -> impl Stream<Item = Verification> {
    let mut check_interval = time::interval(CHECK_INTERVAL);
    check_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    IntervalStream::new(check_interval)
        .filter_map(move |_| {
            let pool_clone = pool.clone();
            let verifs_in_progress = verifs_in_progress.clone();

            async move {
                let in_progress = verifs_in_progress.load(Ordering::Relaxed);
                if in_progress >= MAX_VERIFS_IN_PROGRESS {
                    return None;
                }

                tokio::task::spawn_blocking(move || {
                    let mut conn = pool_clone
                        .get()
                        .expect("Failed to get connection from the pool");
                    Verification::get_pending(&mut conn, MAX_VERIFS_IN_PROGRESS - in_progress)
                })
                .await
                .ok()
            }
        })
        .flat_map(futures::stream::iter)
}

pub async fn run_processor(pool: Arc<Pool>) -> anyhow::Result<JoinHandle<()>> {
    let mut clients = AppClients::default();

    if let Ok(url) = env::var("MAINNET_URL") {
        clients.set(Network::VaraMainnet, Client::new(&url).await?);
    };
    if let Ok(url) = env::var("TESTNET_URL") {
        clients.set(Network::VaraTestnet, Client::new(&url).await?);
    };

    if clients.is_empty() {
        panic!("No network clients are configured");
    }

    let clients = Arc::new(clients);

    let handle = tokio::spawn(async move {
        let in_progress = Arc::new(AtomicI64::new(0));
        let new_verifications = new_verifications(pool.clone(), in_progress.clone());
        futures::pin_mut!(new_verifications);

        loop {
            tokio::select! {
                verif = new_verifications.next() => {
                    process_verif(verif, pool.clone(), in_progress.clone(), clients.clone());
                }
            }
        }
    });

    Ok(handle)
}

fn process_verif(
    verif: Option<Verification>,
    pool: Arc<Pool>,
    in_progress: Arc<AtomicI64>,
    clients: Arc<AppClients>,
) {
    if let Some(verif) = verif {
        in_progress.fetch_add(1, Ordering::Relaxed);

        tokio::spawn(async move {
            let id = verif.id.clone();

            let pool_clone = Arc::clone(&pool);
            let verif_clone = verif.clone();

            if let Err(err) = start_verification(pool_clone.clone(), verif_clone.clone()).await {
                log::warn!("{}: {:?}", &id, err);
            } else if let Err(err) =
                check_code_onchain(clients.clone(), pool_clone.clone(), verif_clone.clone()).await
            {
                log::error!("{}: {:?}", &id, err);
            } else if let Err(err) = build_and_verify(pool_clone.clone(), verif.clone()).await {
                log::error!("{}: {:?}", &id, err);
            }

            in_progress.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

async fn start_verification(pool: Arc<Pool>, verif: Verification) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get connection");
        Verification::update(&mut conn, &verif.id, VerificationStatus::InProgress, None)?;

        // Check if the code is already verified or the verification is in progress
        if Code::get(&mut conn, &verif.code_id).is_some() {
            Verification::update(&mut conn, &verif.id, VerificationStatus::Verified, None)?;
            Err(anyhow!("Code already verified"))
        } else if Verification::is_verification_in_progress(&mut conn, &verif.code_id, &verif.id) {
            Verification::update(&mut conn, &verif.id, VerificationStatus::Pending, None)?;
            Err(anyhow!("Verification in progress"))
        } else {
            Ok(())
        }
    })
    .await?
}

async fn check_code_onchain(
    clients: Arc<AppClients>,
    pool: Arc<Pool>,
    verif: Verification,
) -> Result<()> {
    let Ok(client) = clients.get(&verif.network) else {
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().expect("Failed to get connection");
            Verification::update(
                &mut conn,
                &verif.id,
                VerificationStatus::Failed,
                Some("Unsupported network".into()),
            )
        })
        .await??;
        bail!("Unsupported network");
    };

    if !client.check_code_onchain(verif.code_id.clone()).await? {
        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().expect("Failed to get connection");
            Verification::update(
                &mut conn,
                &verif.id,
                VerificationStatus::Failed,
                Some("Code doesn't exist on chain".into()),
            )
        })
        .await??;
        bail!("Code doesn't exist on chain");
    };

    Ok(())
}

async fn build_and_verify(pool: Arc<Pool>, verif: Verification) -> Result<()> {
    log::info!("{}: building project", &verif.id);
    let build_res = build_project(verif.clone()).await;

    cleanup(&verif.id).await?;

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().expect("Failed to get connection");

        if let Err(err) = build_res {
            let err_msg = format!("Failed to build project. {:?}", err);
            Verification::update(
                &mut conn,
                &verif.id,
                VerificationStatus::Failed,
                Some(err_msg.clone()),
            )?;
            bail!(err_msg);
        }

        let artifacts = build_res.unwrap();

        if artifacts.code_id != verif.code_id {
            Verification::update(
                &mut conn,
                &verif.id,
                VerificationStatus::Failed,
                Some("Code ID mismatch".into()),
            )?;
            bail!(
                "Code ID mismatch. Provided: {}. Calculated: {}",
                &verif.code_id,
                &artifacts.code_id,
            );
        } else {
            let mut idl_hash: Option<String> = None;

            if let Some(idl) = artifacts.idl {
                idl_hash = Some(hash_idl(&idl));
                Idl::save(&mut conn, idl_hash.as_ref().unwrap(), idl)?;
                log::info!("{}: idl saved", &verif.id);
            }
            Code::new(
                &mut conn,
                verif.code_id,
                verif.repo_link,
                artifacts.name,
                idl_hash,
            )?;
            log::info!("{}: code meta saved", &verif.id);
            Verification::update(&mut conn, &verif.id, VerificationStatus::Verified, None)?;
            log::info!("{}: verification completed", &verif.id);
        }

        Ok(())
    })
    .await?
}
