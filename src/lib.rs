pub mod config;
pub mod pki;
pub mod resolver;
pub mod models;
pub mod commands;
#[cfg(test)]
pub mod tests;
pub mod utils;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::fs;
use std::path::Path;
use std::process::Command;
use crate::models::{LinkStatus, DeviceStatus};
use crate::utils::talosctl_wrapper::TalosctlWrapper;

#[async_trait]
pub trait TalosClient: Send + Sync {
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

pub struct CliTalosClient;

#[async_trait]
impl TalosClient for CliTalosClient {
    async fn get_link_status(&self, node_ip: &str) -> Result<Vec<LinkStatus>> {
        let mut links = TalosctlWrapper::get_resource::<LinkStatus>("LinkStatus", node_ip, true)?;
        if links.is_empty() {
             links = TalosctlWrapper::get_resource::<LinkStatus>("LinkStatus", node_ip, false)?;
        }
        Ok(links)
    }

    async fn get_device_status(&self, node_ip: &str) -> Result<Vec<DeviceStatus>> {
        let mut devices = TalosctlWrapper::get_resource::<DeviceStatus>("DeviceStatus", node_ip, true)?;
        if devices.is_empty() {
             devices = TalosctlWrapper::get_resource::<DeviceStatus>("DeviceStatus", node_ip, false)?;
        }
        Ok(devices)
    }

    async fn apply_config(&self, node_ip: &str, file_path: &str, insecure: bool) -> Result<()> {
        TalosctlWrapper::apply_config(node_ip, file_path, insecure)
    }

    async fn validate_config(&self, file_path: &str, mode: &str) -> Result<()> {
        TalosctlWrapper::validate(file_path, mode)
    }

    async fn set_config_endpoint(&self, endpoint: &str) -> Result<()> {
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(["config", "endpoint", endpoint])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Set endpoint failed")) }
    }

    async fn set_config_node(&self, node_fqdn: &str) -> Result<()> {
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(["config", "node", node_fqdn])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Set node failed")) }
    }

    async fn fetch_kubeconfig(&self, node: &str, endpoint: &str, dest: &str) -> Result<()> {
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(["kubeconfig", dest, "--nodes", node, "--endpoints", endpoint, "--force"])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Fetch kubeconfig failed")) }
    }

    async fn bootstrap(&self, node_ip: &str) -> Result<()> {
        for _i in 1..=20 {
            if TalosctlWrapper::bootstrap(node_ip).is_ok() { return Ok(()); }
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
        Err(anyhow!("Bootstrap failed"))
    }

    async fn check_health(&self, endpoint: &str, nodes: &[String]) -> Result<()> {
        TalosctlWrapper::check_health(endpoint, nodes).await
    }

    async fn get_kube_nodes(&self) -> Result<String> {
        let output = Command::new("kubectl")
            .env("KUBECONFIG", "./kubeconfig")
            .args(["get", "nodes", "-o", "wide"])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn get_active_ips(&self) -> Result<Vec<String>> {
        let output = Command::new("nmap")
            .args(["-p", "50000", "-n", "<MGMT_NETWORK_CIDR>", "--exclude", "<GATEWAY_IP>", "--open", "-oG", "-"])
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|l| l.contains("Host:"))
            .filter_map(|l| l.split_whitespace().nth(1))
            .map(|s| s.to_string())
            .collect())
    }

    async fn reset_node(&self, ip: &str) -> Result<()> {
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(["reset", "--nodes", ip, "--force"])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Reset failed")) }
    }

    async fn reset_all(&self) -> Result<()> {
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(["reset", "--all", "--force"])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Reset all failed")) }
    }

    async fn gen_secrets(&self, path: &str) -> Result<()> {
        let status = Command::new("talosctl").args(["gen", "secrets", "-o", path]).status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Gen secrets failed")) }
    }

    async fn gen_config(&self, name: &str, endpoint: &str, secrets_path: &str, output_dir: &str) -> Result<()> {
        let status = Command::new("talosctl")
            .args(["gen", "config", name, endpoint, "--with-secrets", secrets_path, "--output-dir", output_dir, "--force"])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Gen config failed")) }
    }

    async fn patch_config(&self, base_path: &str, patch_path: &str, output_path: &str) -> Result<()> {
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(["machineconfig", "patch", base_path, "--patch", &format!("@{}", patch_path), "-o", output_path])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Patch config failed")) }
    }
}

pub struct UnyeongApp<C: TalosClient> {
    pub config: config::UnyeongConfig,
    pub client: C,
}

impl<C: TalosClient> UnyeongApp<C> {
    pub fn new(config: config::UnyeongConfig, client: C) -> Self {
        Self { config, client }
    }

    pub async fn generate_configs(&self) -> Result<()> {
        println!("🏗️  Generating configurations...");
        if !Path::new("talos/secrets.yaml").exists() {
            self.client.gen_secrets("talos/secrets.yaml").await?;
        }
        self.client.gen_config(&self.config.cluster.name, &self.config.cluster.endpoint, "talos/secrets.yaml", "talos").await?;
        Ok(())
    }

    pub async fn deploy(&self, target_ip: Option<String>) -> Result<()> {
        self.generate_configs().await?;
        if let Some(ip) = target_ip {
            self.deploy_to_ip(&ip).await?;
        } else {
            let active_ips = self.client.get_active_ips().await?;
            for ip in active_ips {
                if let Err(e) = self.deploy_to_ip(&ip).await {
                    eprintln!("❌ Failed to deploy to node {}: {}", ip, e);
                }
            }
        }
        Ok(())
    }

    async fn deploy_to_ip(&self, ip: &str) -> Result<()> {
        let links = self.client.get_link_status(ip).await?;
        for link in links {
            let mac = link.spec.hardware_addr.to_lowercase();
            if let Some((profile, node)) = self.match_profile_and_node(&mac) {
                return self.process_and_deploy(ip, profile, node).await;
            }
        }
        Err(anyhow!("No matching profile/node found for IP {}", ip))
    }

    async fn process_and_deploy(&self, ip: &str, profile: &config::HardwareProfile, node: &config::NodeConfig) -> Result<()> {
        println!("🏷️  Deploying node {} at {}...", node.fqdn, ip);
        let links = self.client.get_link_status(ip).await?;
        let devices = self.client.get_device_status(ip).await?;
        let patch = self.generate_patch(&links, &devices, profile, node, &[(ip.to_string(), node.fqdn.clone())])?;
        let patch_path = format!("talos/{}-patch.yaml", node.fqdn);
        fs::write(&patch_path, patch)?;
        
        let final_config = format!("talos/{}-final.yaml", node.fqdn);
        self.client.patch_config("talos/controlplane.yaml", &patch_path, &final_config).await?;
            
        self.client.apply_config(ip, &final_config, false).await?;
        self.client.bootstrap(ip).await?;
        Ok(())
    }

    pub async fn validate_node_config(&self, ip: &str) -> Result<()> {
        let links = self.client.get_link_status(ip).await?;
        let mut matched = None;
        for link in &links {
            if let Some(res) = self.match_profile_and_node(&link.spec.hardware_addr.to_lowercase()) {
                matched = Some(res);
                break;
            }
        }
        let (profile, node) = matched.ok_or_else(|| anyhow!("No match for IP {}", ip))?;
        let devices = self.client.get_device_status(ip).await?;
        let patch = self.generate_patch(&links, &devices, profile, node, &[(ip.to_string(), node.fqdn.clone())])?;
        let patch_path = format!("talos/{}-validate-patch.yaml", node.fqdn);
        fs::write(&patch_path, patch)?;
        let final_config = format!("talos/{}-validate-final.yaml", node.fqdn);
        self.client.patch_config("talos/controlplane.yaml", &patch_path, &final_config).await?;
        self.client.validate_config(&final_config, "metal").await?;
        Ok(())
    }

    pub async fn verify_cluster_health(&self) -> Result<()> {
        let active_ips = self.client.get_active_ips().await?;
        self.client.check_health(&self.config.cluster.endpoint, &active_ips).await?;
        println!("{}", self.client.get_kube_nodes().await?);
        Ok(())
    }

    pub async fn setup_session_context(&self, node_fqdn: &str) -> Result<()> {
        Command::new("talosctl").args(["config", "merge", "talos/talosconfig"]).status()?;
        Command::new("talosctl").args(["config", "context", &self.config.cluster.name]).status()?;
        self.client.set_config_endpoint(&self.config.cluster.endpoint).await?;
        self.client.set_config_node(node_fqdn).await?;
        Ok(())
    }

    fn match_profile_and_node<'a>(&'a self, mac: &str) -> Option<(&'a config::HardwareProfile, &'a config::NodeConfig)> {
        let profile = self.config.profiles.iter()
            .find(|p| p.matching_macs.iter().any(|m| m.to_lowercase() == mac))?;
        let node = self.config.nodes.iter()
            .find(|n| n.profile == profile.name)?;
        Some((profile, node))
    }

    fn generate_patch(&self, links: &[LinkStatus], devices: &[DeviceStatus], profile: &config::HardwareProfile, node: &config::NodeConfig, all: &[(String, String)]) -> Result<String> {
        let os_disk = resolver::HardwareResolver::resolve_storage(devices, &profile.storage.os_disk)?;
        let nic = resolver::HardwareResolver::resolve_nic(links, &profile.network.endpoint_interface)?;
        let mu_ca = pki::PkiHydrator::get_mu_root_ca().ok();
        let mut patch = format!(r#"
machine:
  type: init
  hostname: {}
  install:
    disk: {}
  network:
    interfaces:
      - interface: {}
        dhcp: true
    extraHostEntries:
"#, node.fqdn, os_disk, nic);

        let mut seen = std::collections::HashSet::new();
        for (ip, fqdn) in all {
            if seen.insert(ip) { patch.push_str(&format!("      - ip: {}\n        aliases:\n          - {}\n", ip, fqdn)); }
        }
        if let Some((ip, _)) = all.first() {
            patch.push_str(&format!("      - ip: {}\n        aliases:\n          - kube.jero-mu.talos.local\n", ip));
        }
        
        if let Some(ca) = mu_ca {
            patch.push_str(&format!(r#"  ca:
    crt: |
      {}
"#, ca));
        }
        Ok(patch)
    }
}
