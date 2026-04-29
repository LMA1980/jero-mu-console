use std::process::Command;
use serde::Deserialize;
use anyhow::Result;

#[derive(Debug, Deserialize)]
struct LinkStatus { metadata: LinkMetadata, spec: LinkSpec }
#[derive(Debug, Deserialize)]
struct LinkMetadata { id: String }
#[derive(Debug, Deserialize)]
struct LinkSpec { #[serde(rename = "hardwareAddr")] hardware_addr: String, #[serde(rename = "speedMbit")] speed_mbit: Option<u32> }

fn main() -> Result<()> {
    for ip in [<NODE1_IP>, <NODE2_IP>] {
        println!("IP: {}", ip);
        let output = Command::new("talosctl").args(["get", "linkstatus", "--nodes", ip, "--insecure", "-o", "json"]).output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Ok(link) = serde_json::from_str::<LinkStatus>(line) {
                println!("ID: {}, MAC: {}, Speed: {:?}", link.metadata.id, link.spec.hardware_addr, link.spec.speed_mbit);
            }
        }
    }
    Ok(())
}
