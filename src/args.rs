use std::path::PathBuf;

use clap::Parser;

use tracing_subscriber::fmt::time::ChronoLocal;

use crate::config;

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    #[clap(long)]
    pub config_path: Option<PathBuf>,
    #[clap(short, long)]
    pub debug: bool,
}

impl Args {
    pub fn init_debug(&self) {
        let mut builder = tracing_subscriber::fmt().with_timer(ChronoLocal::rfc_3339());

        if self.debug {
            builder = builder.with_max_level(tracing::Level::DEBUG);
        } else {
            builder = builder.with_max_level(tracing::Level::INFO);
        }

        builder.init();
    }

    pub fn init_config_path(&self) -> miette::Result<config::SionConfig> {
        match self.config_path {
            Some(ref path) => config::SionConfig::load(&path).map_err(|e| miette::miette!(e)),
            None => {
                let config_path = dirs::config_dir()
                    .expect("failed to get config dir")
                    .join("brain_power/config.kdl");

                if config_path.exists() {
                    return config::SionConfig::load(&config_path)
                        .map_err(|e| miette::miette!(e));
                }

                config::SionConfig::save_default(&config_path)
                    .map_err(|e| miette::miette!(e))?;

                Err(miette::miette!(
                    "no config path provided, default config saved to {}",
                    config_path.display()
                ))
            }
        }
    }
}
