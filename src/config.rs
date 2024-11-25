use std::{
    fs::{
        create_dir_all,
        write,
    },
    path::Path,
};

use miette::{
    miette,
    IntoDiagnostic,
};

#[derive(knuffel::Decode, Debug, PartialEq, Default)]
pub struct SionConfig {
    #[knuffel(child, default)]
    pub gpt: ClientConfig,
    #[knuffel(child, default)]
    pub bot: BotConfig,
}

#[derive(knuffel::Decode, Debug, PartialEq, Default)]
pub struct ClientConfig {
    #[knuffel(child, unwrap(argument), default)]
    pub base_url: String,
    #[knuffel(child, unwrap(argument), default)]
    pub token: String,
}

#[derive(knuffel::Decode, Debug, PartialEq, Default)]
pub struct BotConfig {
    #[knuffel(child, unwrap(argument), default)]
    pub token: String,
    #[knuffel(child, unwrap(argument), default)]
    pub super_user_id: u64,
}

impl SionConfig {
    pub fn load(path: &Path) -> miette::Result<Self> {
        let contents = match std::fs::read_to_string(path).into_diagnostic() {
            Ok(contents) => contents,
            Err(err) => {
                tracing::debug!("failed to read config from {path:?}: {err}");
                return Err(err);
            }
        };

        let config: SionConfig = knuffel::parse(
            path.file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or("config.kdl"),
            &contents,
        )
        .map_err(|e| miette!(e))?;

        tracing::debug!("loaded config from {path:?}");
        Ok(config)
    }

    pub fn save_default(path: &impl AsRef<Path>) -> miette::Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            create_dir_all(parent).into_diagnostic()?;
        }
        let config = include_str!("example.kdl");
        write(path, config).into_diagnostic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() {
        let config: SionConfig = knuffel::parse(
            "test.kdl",
            r#"
            gpt {
                base-url "https://example.com"
                token "token"
            }
            bot {
                token "token"
                super-user-id 123
            }
            "#,
        )
        .unwrap();

        assert_eq!(
            config,
            SionConfig {
                gpt: ClientConfig {
                    base_url: "https://example.com".to_string(),
                    token: "token".to_string(),
                },
                bot: BotConfig {
                    token: "token".to_string(),
                    super_user_id: 123,
                }
            }
        );
    }
}
