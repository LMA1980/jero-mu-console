use clap::{Parser, Subcommand};
use anyhow::Result;
use unyeong::config::UnyeongConfig;
use unyeong::{UnyeongApp, CliTalosClient};
use unyeong::commands::clean::CleanCommand;
use unyeong::commands::dhcp::DhcpCommand;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = "Jero-Mu DevOps Utility")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, default_value = "UnyeongConfig.toml")]
    config: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Clean { #[arg(long)] purge: bool, #[arg(long)] ip: Option<String> },
    Generate,
    Deploy { #[arg(long)] ip: Option<String> },
    Validate { #[arg(long)] ip: String },
    Verify,
    Context { #[arg(long)] node: String },
    Dhcp { #[command(subcommand)] action: DhcpAction },
}

#[derive(Subcommand, Debug)]
enum DhcpAction { Generate }

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = UnyeongConfig::load(&cli.config)?;
    let client = CliTalosClient;
    let app = UnyeongApp::new(config.clone(), CliTalosClient);

    match cli.command {
        Commands::Clean { purge, ip } => CleanCommand::execute(purge, ip, &client).await,
        Commands::Generate => app.generate_configs().await,
        Commands::Deploy { ip } => app.deploy(ip).await,
        Commands::Validate { ip } => app.validate_node_config(&ip).await,

        Commands::Verify => app.verify_cluster_health().await,
        Commands::Context { node } => app.setup_session_context(&node).await,
        Commands::Dhcp { action: DhcpAction::Generate } => DhcpCommand::execute(&config, &client).await,
    }
}
