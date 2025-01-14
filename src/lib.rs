use args::Args;
use clap::Parser;

mod args;
mod bot;
mod cmd;
mod config;
mod models;
mod zero;

fn load_config(args: Args) -> config::SionConfig {
    args.init_config_path().unwrap_or_else(|e| {
        tracing::error!("{}", e);
        std::process::exit(1);
    })
}

pub async fn run() -> anyhow::Result<()> {
    let args = Args::parse();
    args.init_debug();
    let config = load_config(args);

    let bot = bot::Bot::new(config);
    bot.run_active().await
}
