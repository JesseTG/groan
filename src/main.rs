mod types;
mod ai;
mod web;

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use clap::Parser;
use warp::Filter;
use crate::ai::{AiService, ServiceMessage};
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

    let client = Arc::new(Client::with_config(OpenAIConfig::new().with_api_key(cli.key)));

    // Do a basic query just to make sure the key is okay
    let _ = client.models().list().await?;
    // TODO: Make the exit printout look nicer
    // TODO: Validate that the ports aren't equal

    let (sender, receiver) = tokio::sync::mpsc::channel::<ServiceMessage>(32);
    let service = AiService::service(client, Some(sender));

    tokio::join!(
        warp::serve(service).run((cli.ip, cli.port)),
    );

    Ok(())
}
