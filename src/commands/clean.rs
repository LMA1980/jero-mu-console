use anyhow::{Result, anyhow};
use crate::TalosClient;

pub struct CleanCommand;

impl CleanCommand {
    pub async fn execute(purge: bool, target_ip: Option<String>, client: &dyn TalosClient) -> Result<()> {
        if !purge {
            return Err(anyhow!("Purge flag required to reset nodes."));
        }

        if let Some(ip) = target_ip {
            println!("🔥 Nuking node at {}...", ip);
            client.reset_node(&ip).await?;
        } else {
            println!("🔥 Nuking entire cluster...");
            client.reset_all().await?;
        }
        Ok(())
    }
}
