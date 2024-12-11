use anyhow::Result;
use gsdk::{
    metadata::{runtime_types::gear_common::CodeMetadata, storage::GearProgramStorage},
    Api, Value,
};
use hex::FromHex;

use crate::db::Network;

pub struct Client {
    api: Api,
}

impl Client {
    pub async fn new(url: &str) -> Result<Self> {
        let api = Api::new(url).await?;

        Ok(Self { api })
    }

    pub async fn check_code_onchain(&self, code_id: String) -> Result<bool> {
        let code_id = <[u8; 32]>::from_hex(code_id);
        let Ok(code_id) = code_id else {
            return Ok(false);
        };

        let addr = Api::storage(
            GearProgramStorage::MetadataStorage,
            vec![Value::from_bytes(code_id)],
        );

        let result: gsdk::Result<CodeMetadata> = self.api.fetch_storage(&addr).await;

        Ok(result.is_ok())
    }
}

#[derive(Default)]
pub struct AppClients {
    pub mainnet: Option<Client>,
    pub testnet: Option<Client>,
}

impl AppClients {
    pub fn set(&mut self, network: Network, client: Client) {
        match network {
            Network::VaraMainnet => self.mainnet = Some(client),
            Network::VaraTestnet => self.testnet = Some(client),
        }
    }

    pub fn get(&self, network: &Network) -> anyhow::Result<&Client> {
        match network {
            Network::VaraMainnet => {
                if let Some(client) = &self.mainnet {
                    Ok(client)
                } else {
                    Err(anyhow::anyhow!("Mainnet client is not configured"))
                }
            }
            Network::VaraTestnet => {
                if let Some(client) = &self.testnet {
                    Ok(client)
                } else {
                    Err(anyhow::anyhow!("Testnet client is not configured"))
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.mainnet.is_none() && self.testnet.is_none()
    }
}
