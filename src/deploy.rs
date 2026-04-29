use clap::Parser;
use std::process::Command;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;
use serde::Deserialize;
use anyhow::{Result, Context, anyhow};

#[derive(Debug, Deserialize)]
struct DiskStatus {
    metadata: Metadata,
    spec: DiskSpec,
}

#[derive(Debug, Deserialize)]
struct Metadata {
    id: String,
}

#[derive(Debug, Deserialize)]
struct DiskSpec {
    size: u64,
    modalias: Option<String>,
}

const CLUSTER_NAME: &str = "jero-mu";
const ENDPOINT: &str = "https://kube.jero-mu.talos.local:6443";
const NODE_1_DHCP: &str = "<NODE1_IP>";
const NODE_2_DHCP: &str = "<NODE2_IP>";
const NODE_1_MGMT: &str = "<MGMT_IP_NODE1>";

pub fn run() -> Result<()> {
    if !Path::new("talos/secrets.yaml").exists() {
        println!("🔐 Generating secrets...");
        Command::new("talosctl").args(["gen", "secrets", "-o", "talos/secrets.yaml"]).status()?;
    }

    println!("🔍 Discovering hardware on Node 1 ({})...", NODE_1_DHCP);
    let (node1_os, node1_data1, node1_data2) = discover_node1_disks(NODE_1_DHCP)?;
    println!("✅ Node 1 hardware identified.");

    println!("🔍 Discovering hardware on Node 2 ({})...", NODE_2_DHCP);
    let (node2_os, node2_usb) = discover_node2_disks(NODE_2_DHCP)?;
    println!("✅ Node 2 hardware identified.");

    println!("📄 Generating base configs...");
    Command::new("talosctl")
        .args(["gen", "config", CLUSTER_NAME, ENDPOINT, "--with-secrets", "talos/secrets.yaml", "--output-dir", "talos", "--force"])
        .status()?;

    println!("🛠️  Patching and applying node configurations...");
    apply_node_config(1, NODE_1_DHCP, &node1_os, &node1_data1, &node1_data2, "")?;
    apply_node_config(2, NODE_2_DHCP, &node2_os, "", "", &node2_usb)?;

    println!("⏳ Waiting for {}...", NODE_1_MGMT);
    wait_for_talos_api(NODE_1_MGMT)?;

    println!("🏗️  Bootstrapping...");
    Command::new("talosctl").args(["bootstrap", "--nodes", NODE_1_MGMT, "--endpoints", NODE_1_MGMT, "--talosconfig", "talos/talosconfig"]).status()?;

    println!("✨ Done.");
    Ok(())
}

fn discover_node1_disks(ip: &str) -> Result<(String, String, String)> {
    let output = Command::new("talosctl").args(["get", "disks", "--nodes", ip, "--insecure", "-o", "json"]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut disks = Vec::new();
    let stream = serde_json::Deserializer::from_str(&stdout).into_iter::<DiskStatus>();
    for disk in stream {
        let d = disk?;
        if d.spec.size > 0 && !d.metadata.id.contains("loop") { disks.push(d); }
    }
    if disks.len() < 3 { return Err(anyhow!("Node 1 requires 3 disks")); }
    disks.sort_by_key(|d| d.spec.size);
    Ok((disks[0].metadata.id.clone(), disks[disks.len() - 1].metadata.id.clone(), disks[disks.len() - 2].metadata.id.clone()))
}

fn discover_node2_disks(ip: &str) -> Result<(String, String)> {
    let output = Command::new("talosctl").args(["get", "disks", "--nodes", ip, "--insecure", "-o", "json"]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut disks = Vec::new();
    let stream = serde_json::Deserializer::from_str(&stdout).into_iter::<DiskStatus>();
    for disk in stream {
        let d = disk?;
        if d.spec.size > 0 && !d.metadata.id.contains("loop") { disks.push(d); }
    }
    let mut os = String::new();
    let mut usb = String::new();
    for disk in &disks {
        if disk.spec.modalias.as_ref().map(|m| m.contains("usb")).unwrap_or(false) { usb = disk.metadata.id.clone(); }
        else if os.is_empty() { os = disk.metadata.id.clone(); }
    }
    if os.is_empty() { return Err(anyhow!("Node 2 OS disk fail")); }
    if usb.is_empty() && disks.len() > 1 { usb = disks.iter().filter(|d| d.metadata.id != os).map(|d| d.metadata.id.clone()).last().unwrap(); }
    Ok((os, usb))
}

fn apply_node_config(node_id: u8, dhcp_ip: &str, os: &str, d1: &str, d2: &str, usb: &str) -> Result<()> {
    let template_path = format!("talos/node-{}.yaml", node_id);
    let mut patch = fs::read_to_string(&template_path)?;

    patch = patch.replace("DISK_OS", &format!("/dev/{}", os));
    if node_id == 1 {
        patch = patch.replace("DISK_DATA_1", &format!("/dev/{}", d1)).replace("DISK_DATA_2", &format!("/dev/{}", d2));
    } else {
        patch = patch.replace("DISK_DATA_USB", &format!("/dev/{}", usb));
    }

    let final_config = format!("talos/node-{}-final.yaml", node_id);
    Command::new("talosctl")
        .args(["machineconfig", "patch", "talos/controlplane.yaml", "--patch", &patch, "-o", &final_config])
        .status()?;

    Command::new("talosctl").args(["apply-config", "--insecure", "--nodes", dhcp_ip, "--file", &final_config]).status()?;
    Ok(())
}

fn wait_for_talos_api(ip: &str) -> Result<()> {
    for i in 1..=30 {
        let status = Command::new("talosctl").args(["version", "--short", "--nodes", ip, "--endpoints", ip, "--talosconfig", "talos/talosconfig"]).status();
        if let Ok(s) = status { if s.success() { return Ok(()); } }
        println!("Attempt {}: stabilizing...", i);
        thread::sleep(Duration::from_secs(10));
    }
    Err(anyhow!("Timeout {}", ip))
}
