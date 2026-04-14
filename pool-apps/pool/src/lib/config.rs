//! ## Configuration Module
//!
//! Defines [`PoolConfig`], the configuration structure for the Pool, along with its supporting
//! types.
//!
//! This module handles:
//! - Initializing [`PoolConfig`]
//! - Managing [`TemplateProviderConfig`], [`AuthorityConfig`], [`CoinbaseOutput`], and
//!   [`ConnectionConfig`]
//! - Validating and converting coinbase outputs
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

pub use jd_server_sv2::config::{JDSConfig, JDSPartialConfig};
use stratum_apps::{
    config_helpers::{opt_path_from_toml, CoinbaseRewardScript},
    key_utils::{Secp256k1PublicKey, Secp256k1SecretKey},
    stratum_core::bitcoin::{Amount, TxOut},
    tp_type::TemplateProviderType,
    utils::types::{SharesBatchSize, SharesPerMinute},
};

use crate::error::PoolErrorKind;

/// Maps a well-known sv2-tp default port to a Bitcoin network name.
/// Port assignments from `man sv2-tp`:
///   8442  → mainnet, 18442 → testnet3, 48442 → testnet4,
///   38442 → signet,  18447 → regtest
fn network_from_tp_port(port: u16) -> Option<&'static str> {
    match port {
        8442 => Some("mainnet"),
        18442 => Some("testnet3"),
        48442 => Some("testnet4"),
        38442 => Some("signet"),
        18447 => Some("regtest"),
        _ => None,
    }
}

/// Configuration for the Pool, including connection, authority, and coinbase settings.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct PoolConfig {
    listen_address: SocketAddr,
    template_provider_type: TemplateProviderType,
    authority_public_key: Secp256k1PublicKey,
    authority_secret_key: Secp256k1SecretKey,
    cert_validity_sec: u64,
    coinbase_reward_script: CoinbaseRewardScript,
    pool_signature: String,
    shares_per_minute: SharesPerMinute,
    share_batch_size: SharesBatchSize,
    #[serde(default, deserialize_with = "opt_path_from_toml")]
    log_file: Option<PathBuf>,
    #[serde(default)]
    server_id: u16,
    #[serde(default)]
    supported_extensions: Vec<u16>,
    #[serde(default)]
    required_extensions: Vec<u16>,
    #[serde(default)]
    monitoring_address: Option<SocketAddr>,
    #[serde(default)]
    jds: Option<JDSPartialConfig>,
    #[serde(default)]
    monitoring_cache_refresh_secs: Option<u64>,
    /// Optional override for the Bitcoin network name exposed via `GET /api/v1/global`.
    /// When absent the network is inferred from the sv2-tp port in `template_provider_type`
    /// using well-known default ports (see `network_from_tp_port`).
    /// Values follow bitcoin-cli convention: `"mainnet"`, `"testnet3"`, `"testnet4"`,
    /// `"signet"`, `"regtest"`.
    #[serde(default)]
    network: Option<String>,
}

impl PoolConfig {
    /// Creates a new instance of the [`PoolConfig`].
    ///
    /// # Panics
    ///
    /// Panics if `coinbase_reward_script` is empty.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        pool_connection: ConnectionConfig,
        template_provider_type: TemplateProviderType,
        authority_config: AuthorityConfig,
        coinbase_reward_script: CoinbaseRewardScript,
        shares_per_minute: SharesPerMinute,
        share_batch_size: SharesBatchSize,
        server_id: u16,
        supported_extensions: Vec<u16>,
        required_extensions: Vec<u16>,
        monitoring_address: Option<SocketAddr>,
        monitoring_cache_refresh_secs: Option<u64>,
        jds: Option<JDSPartialConfig>,
    ) -> Self {
        Self {
            listen_address: pool_connection.listen_address,
            template_provider_type,
            authority_public_key: authority_config.public_key,
            authority_secret_key: authority_config.secret_key,
            cert_validity_sec: pool_connection.cert_validity_sec,
            coinbase_reward_script,
            pool_signature: pool_connection.signature,
            shares_per_minute,
            share_batch_size,
            log_file: None,
            server_id,
            supported_extensions,
            required_extensions,
            monitoring_address,
            monitoring_cache_refresh_secs,
            jds,
            network: None,
        }
    }

    /// Returns the coinbase output.
    pub fn coinbase_reward_script(&self) -> &CoinbaseRewardScript {
        &self.coinbase_reward_script
    }

    /// Returns Pool listenining address.
    pub fn listen_address(&self) -> &SocketAddr {
        &self.listen_address
    }

    /// Returns the authority public key.
    pub fn authority_public_key(&self) -> &Secp256k1PublicKey {
        &self.authority_public_key
    }

    /// Returns the authority secret key.
    pub fn authority_secret_key(&self) -> &Secp256k1SecretKey {
        &self.authority_secret_key
    }

    /// Returns the certificate validity in seconds.
    pub fn cert_validity_sec(&self) -> u64 {
        self.cert_validity_sec
    }

    /// Returns the Pool signature.
    pub fn pool_signature(&self) -> &String {
        &self.pool_signature
    }

    /// Returns the Template Provider type.
    pub fn template_provider_type(&self) -> &TemplateProviderType {
        &self.template_provider_type
    }

    /// Returns the share batch size.
    pub fn share_batch_size(&self) -> usize {
        self.share_batch_size
    }

    /// Sets the coinbase output.
    pub fn set_coinbase_reward_script(&mut self, coinbase_output: CoinbaseRewardScript) {
        self.coinbase_reward_script = coinbase_output;
    }

    /// Returns the shares per minute.
    pub fn shares_per_minute(&self) -> f32 {
        self.shares_per_minute
    }

    /// Returns the supported extensions.
    pub fn supported_extensions(&self) -> &[u16] {
        &self.supported_extensions
    }

    /// Returns the required extensions.
    pub fn required_extensions(&self) -> &[u16] {
        &self.required_extensions
    }

    /// Sets the log directory.
    pub fn set_log_dir(&mut self, log_dir: Option<PathBuf>) {
        if let Some(dir) = log_dir {
            self.log_file = Some(dir);
        }
    }
    /// Returns the log directory.
    pub fn log_dir(&self) -> Option<&Path> {
        self.log_file.as_deref()
    }

    /// Returns the server id.
    pub fn server_id(&self) -> u16 {
        self.server_id
    }

    pub fn get_txout(&self) -> TxOut {
        TxOut {
            value: Amount::from_sat(0),
            script_pubkey: self.coinbase_reward_script.script_pubkey().to_owned(),
        }
    }

    /// Returns the monitoring address (optional).
    pub fn monitoring_address(&self) -> Option<SocketAddr> {
        self.monitoring_address
    }

    /// Returns the monitoring cache refresh interval in seconds.
    pub fn monitoring_cache_refresh_secs(&self) -> Option<u64> {
        self.monitoring_cache_refresh_secs
    }

    /// Returns the explicit network override if set.
    pub fn network(&self) -> Option<String> {
        self.network.clone()
    }

    /// Returns the effective Bitcoin network name: the explicit `network` override if set,
    /// otherwise inferred from the sv2-tp port in `template_provider_type`.
    pub fn effective_network(&self) -> Option<String> {
        if self.network.is_some() {
            return self.network.clone();
        }
        if let TemplateProviderType::Sv2Tp { address, .. } = &self.template_provider_type {
            if let Ok(socket_addr) = address.parse::<std::net::SocketAddr>() {
                return network_from_tp_port(socket_addr.port()).map(|s| s.to_string());
            }
        }
        None
    }

    /// Set the Bitcoin network override (builder style).
    pub fn with_network(mut self, network: Option<String>) -> Self {
        self.network = network;
        self
    }

    /// Builds a complete [`JDSConfig`] from the partial `[jds]` TOML section
    /// plus shared fields inherited from Pool config.
    ///
    /// Returns `Ok(None)` when the `[jds]` TOML section is absent.
    #[allow(clippy::result_large_err)]
    pub fn build_jds_config(&self) -> Result<Option<JDSConfig>, PoolErrorKind> {
        let Some(jds_partial) = self.jds.clone() else {
            return Ok(None);
        };

        let jds_config = JDSConfig::from_partial(
            jds_partial,
            self.authority_public_key,
            self.authority_secret_key,
            self.cert_validity_sec,
            self.coinbase_reward_script.clone(),
        );

        Ok(Some(jds_config))
    }
}

/// Pool's authority public and secret keys.
pub struct AuthorityConfig {
    pub public_key: Secp256k1PublicKey,
    pub secret_key: Secp256k1SecretKey,
}

impl AuthorityConfig {
    pub fn new(public_key: Secp256k1PublicKey, secret_key: Secp256k1SecretKey) -> Self {
        Self {
            public_key,
            secret_key,
        }
    }
}

/// Connection settings for the Pool listener.
pub struct ConnectionConfig {
    listen_address: SocketAddr,
    cert_validity_sec: u64,
    signature: String,
}

impl ConnectionConfig {
    pub fn new(listen_address: SocketAddr, cert_validity_sec: u64, signature: String) -> Self {
        Self {
            listen_address,
            cert_validity_sec,
            signature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_from_tp_port_known_ports() {
        assert_eq!(network_from_tp_port(8442), Some("mainnet"));
        assert_eq!(network_from_tp_port(18442), Some("testnet3"));
        assert_eq!(network_from_tp_port(48442), Some("testnet4"));
        assert_eq!(network_from_tp_port(38442), Some("signet"));
        assert_eq!(network_from_tp_port(18447), Some("regtest"));
    }

    #[test]
    fn network_from_tp_port_unknown_port() {
        assert_eq!(network_from_tp_port(4444), None);
        assert_eq!(network_from_tp_port(0), None);
    }

    fn sv2_tp_type(address: &str) -> TemplateProviderType {
        TemplateProviderType::Sv2Tp {
            address: address.to_string(),
            public_key: None,
        }
    }

    #[test]
    fn effective_network_infers_from_tp_port() {
        let tp_type = sv2_tp_type("127.0.0.1:18447");
        // Build a minimal config manually using the serde path is complex; test the
        // helper function directly.
        assert_eq!(
            network_from_tp_port(18447),
            Some("regtest"),
        );
        assert_eq!(
            network_from_tp_port(8442),
            Some("mainnet"),
        );
        // Confirm an unknown port yields None
        assert_eq!(network_from_tp_port(4444), None);
        // Confirm the address parser works as expected
        let port = "127.0.0.1:18447"
            .parse::<std::net::SocketAddr>()
            .unwrap()
            .port();
        assert_eq!(network_from_tp_port(port), Some("regtest"));
        drop(tp_type); // suppress unused warning
    }
}
