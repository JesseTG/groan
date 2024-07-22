use clap::Parser;

// NOTE: These doc comments are parsed and embedded into the CLI itself.

/// groan - Good RetroArch OpenAI iNtegration
#[derive(Parser, Debug)]
#[command(version, about, long_about)]
struct Cli {
    /// The API key used to authenticate with OpenAI.
    /// Provide on the command-line or with the OPENAI_API_KEY environment variable.
    #[arg(short, long, env="OPENAI_API_KEY")]
    key: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    println!("{}", cli.key);

    // TODO: Serve up "hello world" in-browser
}
