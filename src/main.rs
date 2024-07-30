mod types;
mod service;

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use clap::Parser;
use warp::Filter;
use bytes::Bytes;
use crate::service::query_service;
use crate::types::{InvalidRequestBody, RequestBody, RequestParams};
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
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    pretty_env_logger::init();

    let client = Arc::new(Client::with_config(OpenAIConfig::new().with_api_key(cli.key)));

    // Do a basic query just to make sure the key is okay
    let _ = client.models().list().await?;
    // TODO: Make the exit printout look nicer

    let service = warp::post() // Accept only POST requests...
        // ...at the root path...
        .and(warp::path::end())
        // ...with query parameters that suit RequestParams...
        .and(warp::query::<RequestParams>())
        // ...regardless of the declared content type.
        .and(warp::body::bytes())
        // RetroArch declares application/x-www-form-urlencoded for its AI service requests,
        // but the body is actually JSON;
        // hence we deserialize explicitly because warp doesn't know how to handle this discrepancy.
        .and_then(|params, body: Bytes| async move {
            log::info!(target: "groan", "{:?}", params);
            if let Ok(body) = serde_json::from_slice::<RequestBody>(body.iter().as_slice()) {
                log::info!(target: "groan", "{:?}", body);
                Ok((params, body))
            } else {
                Err(warp::reject::custom(InvalidRequestBody))
            }
        })
        .untuple_one()
        // query_service may run on another thread, possibly with multiple instances;
        // therefore we create the client in an `Arc` and clone it for each call to this endpoint
        .then(move |params, body| query_service(Arc::clone(&client), params, body))
        .map(|response| warp::reply::json(&response))
        .with(warp::trace::named("groan"));

    warp::serve(service)
        .run((cli.ip, cli.port))
        .await;

    Ok(())
}
