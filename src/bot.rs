use reqwest::Client;
use teloxide::{
    dispatching::UpdateFilterExt,
    prelude::*,
    types::{
        Me,
        Message,
        UserId,
    },
    utils::command::BotCommands,
};

use crate::{
    cmd::{
        self,
        Command,
    },
    config::SionConfig,
    zero,
};

const GENERATING_HINT: &str = "Generating...";

#[derive(Debug, Clone)]
pub struct Bot {
    bot: teloxide::Bot,
    super_user_id: UserId,
    zero_client: zero::SionClient,
}

impl Bot {
    pub fn new(config: SionConfig) -> Self {
        let http_client = Client::builder()
            .https_only(true)
            .http2_adaptive_window(true)
            .build()
            .expect("failed to build http client");

        let bot = teloxide::Bot::with_client(config.bot.token, http_client);

        Self {
            bot,
            super_user_id: UserId(config.bot.super_user_id),
            zero_client: zero::SionClient::new(config.gpt),
        }
    }

    async fn handle_command(&self, msg: Message, me: Me) -> anyhow::Result<()> {
        let Some(cmd) = msg
            .text()
            .and_then(|msg| cmd::Command::parse(msg, me.username()).ok())
        else {
            return Ok(());
        };

        if msg.from.as_ref().map(|user| user.id) != Some(self.super_user_id) {
            tracing::error!("ignore message from non-super user");
            self.bot
                .send_message(msg.chat.id, "I don't know you.")
                .await?;

            return Ok(());
        }

        match cmd {
            Command::Help => self.handle_help_request(msg).await?,
            Command::Meow(prompt) => self.handle_prompt(msg, prompt).await?,
        }

        Ok(())
    }

    async fn handle_help_request(&self, msg: Message) -> anyhow::Result<()> {
        self.bot
            .send_message(msg.chat.id, Command::descriptions().to_string())
            .await?;

        tracing::info!("send help done");
        Ok(())
    }

    async fn handle_prompt(&self, msg: Message, prompt: String) -> anyhow::Result<()> {
        if prompt.is_empty() {
            self.bot.send_message(msg.chat.id, "什么都没有!").await?;

            return Ok(());
        }

        let gen_msg = self.bot.send_message(msg.chat.id, GENERATING_HINT).await?;

        let hint = match self.zero_client.request_new_hint(prompt).await {
            Ok(hint) => hint,
            Err(e) => {
                tracing::error!("failed to generate hint: {e}");
                format!("Failed to generate hint: {e}")
            }
        };
        self.bot
            .edit_message_text(msg.chat.id, gen_msg.id, hint)
            .await?;
        tracing::info!("handle text message");
        Ok(())
    }

    pub async fn run_active(&self) -> anyhow::Result<()> {
        let handler = dptree::entry().branch(Update::filter_message().endpoint({
            let bot = self.clone();

            move |_: teloxide::Bot, msg: Message, me: Me| {
                let bot = bot.clone();

                async move { bot.handle_command(msg, me).await }
            }
        }));

        tracing::info!("Bot is running...");

        Dispatcher::builder(self.bot.clone(), handler)
            .build()
            .dispatch()
            .await;

        Err(anyhow::anyhow!("Bot is stopped"))
    }
}
