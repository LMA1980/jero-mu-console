use serde::Deserialize;
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct UnyeongConfig {
    pub cluster: ClusterConfig,
    pub profiles: Vec<HardwareProfile>,
    pub nodes: Vec<NodeConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClusterConfig {
    pub name: String,
    pub endpoint: String,
    pub global_sans: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HardwareProfile {
    pub name: String,
    pub matching_macs: Vec<String>,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub endpoint_interface: String, // Rule or name
    pub listen_interfaces: Vec<String>,
    pub ignore_offline: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
    pub os_disk: StorageRule,
    pub data_disks: Vec<StorageRule>,
    pub ignore_disks: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageRule {
    pub disk_type: Option<String>, // "nvme", "ssd", "hdd"
    pub min_size: Option<String>,
    pub index: Option<usize>,
    pub path: Option<String>, // Explicit override
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub fqdn: String,
    pub profile: String,
    pub network_mode: Option<String>, // e.g. "dhcpv4"
    pub network_override: Option<NetworkConfig>,
    pub storage_override: Option<StorageConfig>,
}

impl UnyeongConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: UnyeongConfig = toml::from_str(&content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parse() {
        let toml_str = r#"
[cluster]
name = "test-cluster"
endpoint = "https://endpoint"
global_sans = ["san1"]

[[profiles]]
name = "profile1"
matching_macs = ["mac1"]
[profiles.network]
endpoint_interface = "eno1"
listen_interfaces = ["eno1"]
ignore_offline = true
[profiles.storage]
os_disk = { disk_type = "ssd" }
data_disks = []
ignore_disks = []

[[nodes]]
fqdn = "node1"
profile = "profile1"
"#;
        let config: UnyeongConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.cluster.name, "test-cluster");
        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.nodes.len(), 1);
    }
}
