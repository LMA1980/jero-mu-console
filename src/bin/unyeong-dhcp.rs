use anyhow::Result;
use unyeong::config::UnyeongConfig;
use unyeong::commands::dhcp::DhcpCommand;
use unyeong::CliTalosClient;

#[tokio::main]
async fn main() -> Result<()> {
    let config = UnyeongConfig::load("UnyeongConfig.toml")?;
    let client = CliTalosClient;
    DhcpCommand::execute(&config, &client).await
}
