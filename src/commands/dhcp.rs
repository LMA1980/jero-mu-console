use anyhow::Result;
use crate::TalosClient;
use crate::config::UnyeongConfig;
use std::process::Command;

pub struct DhcpCommand;

impl DhcpCommand {
    pub async fn execute(config: &UnyeongConfig, client: &dyn TalosClient) -> Result<()> {
        let active_ips = client.get_active_ips().await?;
        println!("# --- talos.local ---");
        
        let mut node_ips: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        // Group discovered IPs by their matched Node FQDN
        for ip in active_ips {
            if let Ok(links) = client.get_link_status(&ip).await {
                for link in links {
                    let mac = link.spec.hardware_addr.to_lowercase();
                    if let Some(node) = config.nodes.iter().find(|n| 
                        config.profiles.iter().find(|p| p.name == n.profile)
                            .map(|p| p.matching_macs.iter().any(|m| m.to_lowercase() == mac))
                            .unwrap_or(false)
                    ) {
                        node_ips.entry(node.fqdn.clone()).or_default().push(ip.clone());
                        break;
                    }
                }
            }
        }

        // Output cleaned mapping
        for (fqdn, ips) in node_ips {
            let mut unique_ips = ips;
            unique_ips.sort();
            unique_ips.dedup();

            for (i, ip) in unique_ips.iter().enumerate() {
                if i == 0 {
                    println!("{}    {} kube.jero-mu.talos.local", ip, fqdn);
                } else {
                    println!("{}    {}", ip, fqdn);
                }
            }
        }
        println!("# --- end talos.local ---");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::MockTalosClient;
    use crate::config::{ClusterConfig, HardwareProfile, NodeConfig, NetworkConfig, StorageConfig, StorageRule, UnyeongConfig};
    use crate::models::{LinkMetadata, LinkSpec, LinkStatus};

    #[tokio::test]
    async fn test_dhcp_mapping() {
        let mut mock = MockTalosClient::new();
        let config = UnyeongConfig {
            cluster: ClusterConfig { name: "test".into(), endpoint: "https://test".into(), global_sans: vec![] },
            profiles: vec![HardwareProfile {
                name: "p1".into(),
                matching_macs: vec!["mac1".into()],
                network: NetworkConfig { endpoint_interface: "eno1".into(), listen_interfaces: vec![], ignore_offline: true },
                storage: StorageConfig { os_disk: StorageRule { path: None, disk_type: None, min_size: None, index: None }, data_disks: vec![], ignore_disks: vec![] },
            }],
            nodes: vec![NodeConfig { fqdn: "node1".into(), profile: "p1".into(), network_mode: None, network_override: None, storage_override: None }],
        };

        mock.expect_get_active_ips()
            .returning(|| Ok(vec!["<TEST_IP>".into()]));

        mock.expect_get_link_status()
            .returning(|_| Ok(vec![LinkStatus {
                metadata: LinkMetadata { id: "eth0".into() },
                spec: LinkSpec { hardware_addr: "mac1".into() },
            }]));

        assert!(DhcpCommand::execute(&config, &mock).await.is_ok());
    }
}
