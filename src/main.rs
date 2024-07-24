mod types;

use std::net::{IpAddr, Ipv4Addr};
use clap::Parser;
use warp::Filter;
use bytes::Bytes;
use crate::types::{InvalidRequestBody, OutputFormat, RequestBody, RequestParams};
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

async fn query_service(params: RequestParams, body: RequestBody) -> String {
    match params.output.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice() {
        ["text", ..] => {
            return format!("text");
        },
        ["sound", "wav", ..] => {
            return "sound: wav".to_string();
        },
        ["image", "png", "png-a", ..] => {
            return format!("image png");
        },
        _ => {
            return "unknown".to_string();
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    pretty_env_logger::init();

    let hello = warp::post() // Accept only POST requests...
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
                Ok((params, body))
            } else {
                Err(warp::reject::custom(InvalidRequestBody))
            }
        })
        .untuple_one()
        .then(query_service)
        .with(warp::trace::named("groan"));

    warp::serve(hello)
        .run((cli.ip, cli.port))
        .await;
}
