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

type Blake2b256 = Blake2b<U32>;

pub fn generate_code_id(code: &[u8]) -> String {
    let mut hasher = Blake2b256::new();

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

pub fn validate_and_get_code_id(code_id: &str) -> Result<String> {
    let code_id = code_id.strip_prefix("0x").unwrap_or(code_id);

    if code_id.len() != 64 {
        bail!("Invalid code ID");
    }

    Ok(code_id.to_string())
}
