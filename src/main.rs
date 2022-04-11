use anyhow::Result;
use clap::Parser;
use sps::config::must_load;
use sps::service::Context;

/// Simple push service
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file
    #[clap(short, long, default_value = "etc/sps.toml")]
    config: String,
}

#[async_std::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let c = must_load(&args.config).await;

    let _ctx = Context::new(&c).await?;
    Ok(())
}
