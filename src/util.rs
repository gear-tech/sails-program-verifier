use crate::consts::AVAILABLE_VERSIONS;
use anyhow::{bail, Result};
use blake2::{digest::typenum::U32, Blake2b, Digest};
use rand::{self, distributions::Alphanumeric, thread_rng, Rng};

pub fn generate_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect::<String>()
}

pub fn generate_code_id(code: &[u8]) -> String {
    let mut hasher = Blake2b::<U32>::new();

    hasher.update(code);
    hex::encode(hasher.finalize().as_slice())
}

pub fn hash_idl(idl_data: &str) -> String {
    let mut hasher = Blake2b::<U32>::new();

    hasher.update(idl_data.as_bytes());
    hex::encode(hasher.finalize().as_slice())
}

pub fn check_docker_version(version: &str) -> Result<()> {
    if AVAILABLE_VERSIONS.contains(&version) {
        Ok(())
    } else {
        bail!(
            "Unsupported docker version. Available versions: {:?}",
            AVAILABLE_VERSIONS
        )
    }
}
