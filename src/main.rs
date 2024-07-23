mod types;
use clap::Parser;
use warp::Filter;
use crate::types::RequestParams;
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
