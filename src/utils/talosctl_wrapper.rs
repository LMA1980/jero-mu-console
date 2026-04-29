use std::process::Command;
use anyhow::{Result, anyhow};
use serde::de::DeserializeOwned;

pub struct TalosctlWrapper;

impl TalosctlWrapper {
    pub fn get_resource<T: DeserializeOwned>(resource: &str, node_ip: &str, insecure: bool) -> Result<Vec<T>> {
        let mut args = vec!["get", resource, "--nodes", node_ip, "--endpoints", node_ip, "-o", "json"];
        if insecure {
            args.push("--insecure");
        }

        let output = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(&args)
            .output()?;

        if !output.status.success() {
            return Err(anyhow!("Failed to execute talosctl get {}", resource));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stream = serde_json::Deserializer::from_str(&stdout).into_iter::<T>();
        Ok(stream.filter_map(|item| item.ok()).collect())
    }

    pub fn apply_config(node_ip: &str, file_path: &str, insecure: bool) -> Result<()> {
        let mut args = vec!["apply-config", "--nodes", node_ip, "--file", file_path];
        if insecure {
            args.push("--insecure");
        }
        
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(&args)
            .status()?;
            
        if status.success() { Ok(()) } else { Err(anyhow!("Apply config failed")) }
    }

    pub fn bootstrap(node_ip: &str) -> Result<()> {
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(["bootstrap", "--nodes", node_ip, "--endpoints", node_ip])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Bootstrap failed")) }
    }

    pub fn validate(file_path: &str, mode: &str) -> Result<()> {
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args(["validate", "--config", file_path, "--mode", mode])
            .status()?;
        if status.success() { Ok(()) } else { Err(anyhow!("Validation failed")) }
    }

    pub async fn check_health(endpoint: &str, nodes: &[String]) -> Result<()> {
        let first_node = nodes.first().ok_or_else(|| anyhow!("No nodes provided for health check"))?;
        let status = Command::new("talosctl")
            .env("TALOSCONFIG", "talos/talosconfig")
            .args([
                "health",
                "--endpoints",
                endpoint,
                "--nodes",
                first_node,
                "--wait-timeout",
                "10m",
            ])
            .status()?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("Cluster is NOT healthy"))
        }
    }
}
