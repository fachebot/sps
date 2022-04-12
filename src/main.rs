use anyhow::Result;
use clap::Parser;
use sps::config;
use sps::handler;
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
    let c = config::must_load(&args.config).await;

    let ctx: Context = Context::new(&c).await?;
    let mut app = tide::Server::with_state(ctx);
    handler::register_handlers(&mut app)?;

    let addr = format!("0.0.0.0:{}", c.server.port);
    println!("Starting server at {}...", addr);
    app.listen(addr).await?;

    Ok(())
}
