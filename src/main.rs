use anyhow::Result;
use chrono::Datelike;
use clap::Parser;
use simplelog::*;
use sps::config;
use sps::handler;
use sps::job;
use sps::service::Context;
use std::fs::File;
use std::path::Path;

/// Simple push service
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Configuration file
    #[clap(short, long, default_value = "etc/sps.toml")]
    config: String,
}

fn init_logger() {
    let dir = Path::new("logs");
    if !dir.exists() {
        std::fs::create_dir_all(dir).unwrap();
    }

    let now = chrono::Utc::now();
    let filename = format!("{}-{}-{}.log", now.year(), now.month(), now.day());

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(format!("logs/{}", filename)).unwrap(),
        ),
    ])
    .unwrap();
}

#[async_std::main]
async fn main() -> Result<()> {
    init_logger();

    let args = Args::parse();
    let c = config::must_load(&args.config).await;

    let ctx = Context::make_pointer(&c).await?;
    let pusher = job::Pusher::new(ctx.clone(), 12);
    pusher.start()?;

    let tg_bot = job::TelegramBot::new(ctx.clone());
    tg_bot.start()?;

    let mut app = tide::Server::with_state(ctx.clone());
    handler::register_handlers(&mut app)?;

    let addr = format!("0.0.0.0:{}", c.server.port);
    println!("Starting server at {}...", addr);
    app.listen(addr).await?;

    Ok(())
}
