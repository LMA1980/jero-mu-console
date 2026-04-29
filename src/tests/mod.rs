use crate::*;
use crate::config::{ClusterConfig, HardwareProfile, NetworkConfig, NodeConfig, StorageConfig, StorageRule, UnyeongConfig};
use crate::models::{LinkStatus, DeviceStatus, LinkMetadata, LinkSpec};
use anyhow::Result;
use async_trait::async_trait;

mockall::mock! {
    pub TalosClient {}
    #[async_trait]
    impl TalosClient for TalosClient {
        async fn get_link_status(&self, node_ip: &str) -> Result<Vec<LinkStatus>>;
        async fn get_device_status(&self, node_ip: &str) -> Result<Vec<DeviceStatus>>;
        async fn apply_config(&self, node_ip: &str, file_path: &str, insecure: bool) -> Result<()>;
        async fn validate_config(&self, file_path: &str, mode: &str) -> Result<()>;
        async fn set_config_endpoint(&self, endpoint: &str) -> Result<()>;
        async fn set_config_node(&self, node_fqdn: &str) -> Result<()>;
        async fn fetch_kubeconfig(&self, node: &str, endpoint: &str, dest: &str) -> Result<()>;
        async fn bootstrap(&self, node_ip: &str) -> Result<()>;
        async fn check_health(&self, endpoint: &str, nodes: &[String]) -> Result<()>;
        async fn get_kube_nodes(&self) -> Result<String>;
        async fn get_active_ips(&self) -> Result<Vec<String>>;
        async fn reset_node(&self, ip: &str) -> Result<()>;
        async fn reset_all(&self) -> Result<()>;
        async fn gen_secrets(&self, path: &str) -> Result<()>;
        async fn gen_config(&self, name: &str, endpoint: &str, secrets_path: &str, output_dir: &str) -> Result<()>;
        async fn patch_config(&self, base_path: &str, patch_path: &str, output_path: &str) -> Result<()>;
    }
}

fn create_test_config() -> UnyeongConfig {
    UnyeongConfig {
        cluster: ClusterConfig {
            name: "test".into(),
            endpoint: "https://test".into(),
            global_sans: vec![],
        },
        profiles: vec![HardwareProfile {
            name: "p1".into(),
            matching_macs: vec!["mac1".into()],
            network: NetworkConfig {
                endpoint_interface: "eno1".into(),
                listen_interfaces: vec![],
                ignore_offline: true,
            },
            storage: StorageConfig {
                os_disk: StorageRule {
                    path: Some("/dev/sda".into()),
                    disk_type: None,
                    min_size: None,
                    index: None,
                },
                data_disks: vec![],
                ignore_disks: vec![],
            },
        }],
        nodes: vec![NodeConfig {
            fqdn: "node1".into(),
            profile: "p1".into(),
            network_mode: None,
            network_override: None,
            storage_override: None,
        }],
    }
}

#[tokio::test]
async fn test_setup_session_context() {
    let mut mock = MockTalosClient::new();
    let config = create_test_config();

    mock.expect_set_config_endpoint().returning(|_| Ok(()));
    mock.expect_set_config_node().returning(|_| Ok(()));

    let app = UnyeongApp::new(config, mock);
    app.setup_session_context("node1").await.unwrap();
}

#[tokio::test]
async fn test_verify_cluster_health() {
    let mut mock = MockTalosClient::new();
    let config = create_test_config();

    mock.expect_get_active_ips().returning(|| Ok(vec!["127.0.0.1".into()]));
    mock.expect_check_health().returning(|_, _| Ok(()));
    mock.expect_get_kube_nodes().returning(|| Ok("Ready".into()));

    let app = UnyeongApp::new(config, mock);
    app.verify_cluster_health().await.unwrap();
}

#[tokio::test]
async fn test_deploy_surgical() {
    let mut mock = MockTalosClient::new();
    let config = create_test_config();

    mock.expect_get_link_status().returning(|_| Ok(vec![LinkStatus {
        metadata: LinkMetadata { id: "eno1".into() },
        spec: LinkSpec { hardware_addr: "mac1".into() },
    }]));
    mock.expect_get_device_status().returning(|_| Ok(vec![]));
    mock.expect_patch_config().returning(|_, _, _| Ok(()));
    mock.expect_apply_config().returning(|_, _, _| Ok(()));
    mock.expect_bootstrap().returning(|_| Ok(()));

    let app = UnyeongApp::new(config, mock);
    // Since generate_configs calls Command directly, we skip full deploy test but test deploy_to_ip
    app.deploy_to_ip("127.0.0.1").await.unwrap();
}

#[tokio::test]
async fn test_validate_node_config() {
    let mut mock = MockTalosClient::new();
    let config = create_test_config();

    mock.expect_get_link_status().returning(|_| Ok(vec![LinkStatus {
        metadata: LinkMetadata { id: "eno1".into() },
        spec: LinkSpec { hardware_addr: "mac1".into() },
    }]));
    mock.expect_get_device_status().returning(|_| Ok(vec![]));
    mock.expect_patch_config().returning(|_, _, _| Ok(()));
    mock.expect_validate_config().returning(|_, _| Ok(()));

    let app = UnyeongApp::new(config, mock);
    app.validate_node_config("127.0.0.1").await.unwrap();
}

#[tokio::test]
async fn test_generate_configs() {
    let mut mock = MockTalosClient::new();
    let config = create_test_config();

    mock.expect_gen_secrets().returning(|_| Ok(()));
    mock.expect_gen_config().returning(|_, _, _, _| Ok(()));

    let app = UnyeongApp::new(config, mock);
    app.generate_configs().await.unwrap();
}

#[tokio::test]
async fn test_clean_command() {
    let mut mock = MockTalosClient::new();
    mock.expect_reset_node().returning(|_| Ok(()));
    mock.expect_reset_all().returning(|| Ok(()));

    assert!(crate::commands::clean::CleanCommand::execute(true, Some("127.0.0.1".into()), &mock).await.is_ok());
    assert!(crate::commands::clean::CleanCommand::execute(true, None, &mock).await.is_ok());
    assert!(crate::commands::clean::CleanCommand::execute(false, None, &mock).await.is_err());
}

#[test]
fn test_match_profile_and_node() {
    let config = create_test_config();
    let client = MockTalosClient::new();
    let app = UnyeongApp::new(config, client);
    let (p, n) = app.match_profile_and_node("mac1").unwrap();
    assert_eq!(p.name, "p1");
    assert_eq!(n.fqdn, "node1");
}

#[test]
fn test_config_load() {
    let toml_str = r#"
[cluster]
name = "test"
endpoint = "test"
global_sans = []
[[profiles]]
name = "p1"
matching_macs = ["mac1"]
[profiles.network]
endpoint_interface = "eno1"
listen_interfaces = []
ignore_offline = true
[profiles.storage]
os_disk = { path = "/dev/sda" }
data_disks = []
ignore_disks = []
[[nodes]]
fqdn = "node1"
profile = "p1"
"#;
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("UnyeongConfig.toml");
    std::fs::write(&file_path, toml_str).unwrap();
    
    let config = UnyeongConfig::load(&file_path).unwrap();
    assert_eq!(config.cluster.name, "test");
}
