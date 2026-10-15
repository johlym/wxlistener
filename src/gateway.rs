//! Gateway transport: the local HTTP API, or the legacy TCP client.

use anyhow::Result;
use std::collections::HashMap;

use crate::client::GW1000Client;
use crate::config::GatewaySettings;
use crate::http_gateway::HttpGateway;

/// Live-data source shared by the poll loop and the web server.
#[derive(Clone)]
pub enum Gateway {
    Legacy(GW1000Client),
    Http(HttpGateway),
}

impl Gateway {
    pub fn connect(settings: &GatewaySettings) -> Result<Self> {
        if settings.legacy {
            Ok(Self::Legacy(GW1000Client::new(
                settings.ip.clone(),
                settings.port,
            )))
        } else {
            Ok(Self::Http(HttpGateway::new(
                settings.ip.clone(),
                settings.port,
                settings.rain_source,
            )?))
        }
    }

    pub fn get_firmware_version(&self) -> Result<String> {
        match self {
            Self::Legacy(client) => client.get_firmware_version(),
            Self::Http(client) => client.get_firmware_version(),
        }
    }

    pub fn get_mac_address(&self) -> Result<String> {
        match self {
            Self::Legacy(client) => client.get_mac_address(),
            Self::Http(client) => client.get_mac_address(),
        }
    }

    pub fn get_livedata(&self) -> Result<HashMap<String, f64>> {
        match self {
            Self::Legacy(client) => client.get_livedata(),
            Self::Http(client) => client.get_livedata(),
        }
    }
}

/// Run a blocking gateway call without holding a Tokio worker.
pub async fn blocking_gateway<T, F>(gateway: Gateway, call: F) -> Result<T>
where
    T: Send + 'static,
    F: FnOnce(&Gateway) -> Result<T> + Send + 'static,
{
    tokio::task::spawn_blocking(move || call(&gateway))
        .await
        .map_err(|err| anyhow::anyhow!("gateway task failed: {err}"))?
}
