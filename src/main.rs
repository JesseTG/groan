use clap::Parser;
use serde::{Deserialize, Serialize};
use warp::Filter;

// NOTE: These doc comments are parsed and embedded into the CLI itself.

/// groan - Good RetroArch OpenAI iNtegration
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// The API key used to authenticate with OpenAI.
    /// Provide on the command-line or with the OPENAI_API_KEY environment variable.
    #[arg(short, long, env = "OPENAI_API_KEY")]
    key: String,

    #[arg(short, long, default_value_t = 4404)]
    port: u16,

    // TODO: Select a host
}

#[derive(Deserialize, Serialize, Debug)]
struct RequestParams {
    source_lang: Option<String>,
    target_lang: Option<String>,
    output: String,
}

#[tokio::main]
async fn main() {

    let cli = Cli::parse();
    pretty_env_logger::init();

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path::end() // Only match the root
        .and(warp::query::<RequestParams>())
        .map(|name| format!("Hello, {:?}!", name))
        .with(warp::log("groan::api"));

    warp::serve(hello)
        .run(([127, 0, 0, 1], cli.port))
        .await;
}
