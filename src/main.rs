mod ai;
mod types;
mod web;

use crate::ai::AiService;
use crate::web::WebConsoleService;
use async_openai::config::OpenAIConfig;
use async_openai::Client;
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
// NOTE: These doc comments are parsed and embedded into the CLI itself.

/// groan - Good RetroArch OpenAI iNtegration
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// The API key used to authenticate with OpenAI.
    /// Provide on the command-line or with the OPENAI_API_KEY environment variable.
    #[arg(short, long, env = "OPENAI_API_KEY")]
    key: String,

    #[arg(short, long, default_value_t = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))]
    ip: IpAddr,

    #[arg(short, long, default_value_t = 4404)]
    port: u16,

    // TODO: Allow the console to bind on a separate interface
    #[arg(short, long, default_value_t = 4405)]
    console_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    pretty_env_logger::init();

    let client = Arc::new(Client::with_config(
        OpenAIConfig::new().with_api_key(cli.key),
    ));

    // Do a basic query just to make sure the key is okay
    let _ = client.models().list().await?;
    // TODO: Make the exit printout look nicer
    // TODO: Validate that the ports aren't equal

    let (sender, receiver) = tokio::sync::mpsc::channel(32);
    let ai_service = AiService::service(client, Some(sender));
    let mut web_service = WebConsoleService::new();

    tokio::join!(
        warp::serve(ai_service).run((cli.ip, cli.port)),
        warp::serve(WebConsoleService::server_filter()).run((cli.ip, cli.console_port)),
        web_service.poll_task(receiver),
    );

    Ok(())
}
