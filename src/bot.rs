use std::sync::Arc;

use reqwest::Client;
use teloxide::{
    dispatching::UpdateFilterExt,
    prelude::*,
    types::{
        InlineQueryResult,
        InlineQueryResultArticle,
        InputMessageContent,
        InputMessageContentText,
        Me,
        Message,
        UserId,
    },
    utils::command::BotCommands,
};
use tokio::sync::Mutex;

use crate::{
    cmd::{
        self,
        Command,
    },
    config::SionConfig,
    models::Model,
    zero,
};

const GENERATING_HINT: &str = "Generating...";

#[derive(Debug, Clone)]
pub struct Bot {
    bot: teloxide::Bot,
    super_user_id: UserId,
    zero_client: zero::SionClient,
    model: Arc<Mutex<Model>>,
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
            model: Arc::new(Mutex::new(Model::default())),
        }
    }

    async fn handle_command(&self, msg: Message, me: Me) -> anyhow::Result<()> {
        let Some(cmd) = msg
            .text()
            .and_then(|msg| cmd::Command::parse(msg, me.username()).ok())
        else {
            return Ok(());
        };

        if let Some(user) = msg.from.as_ref() {
            if self.super_user_id != user.id {
                tracing::error!("ignore message from non-super user");

                self.bot
                    .send_message(msg.chat.id, "You are not my master!")
                    .await?;

                self.bot
                    .send_message(
                        self.super_user_id,
                        format!(
                            "Someone tried to use me! \nUserId: {}\nUsername: {}\nLink: {}",
                            user.id,
                            user.username.clone().unwrap_or("None".to_string()),
                            user.url()
                        ),
                    )
                    .await?;

                return Ok(());
            }
        } else {
            tracing::error!("ignore message from non-super user");
            return Ok(());
        }

        match cmd {
            Command::Help => self.handle_help_request(msg).await?,
            Command::Meow(prompt) => self.handle_prompt(msg, prompt).await?,
            Command::Model(model) => self.handle_model(msg, model).await?,
            Command::LookModel => self.handle_look_model(msg).await?,
        }

        Ok(())
    }

    async fn handle_inline(&self, q: InlineQuery) -> anyhow::Result<()> {
        let msg = if self.super_user_id != q.from.id {
            tracing::info!("ignore message from non-super user");
            InlineQueryResultArticle::new(
                "01".to_string(),
                "You are not my master!",
                InputMessageContent::Text(InputMessageContentText::new(
                    "You are not my master!",
                )),
            )
        } else if !q.query.ends_with("喵") {
            tracing::info!("query does not end with 喵");
            InlineQueryResultArticle::new(
                "01".to_string(),
                "Please end your query with 喵",
                InputMessageContent::Text(InputMessageContentText::new(
                    "Please end your query with 喵",
                )),
            )
        } else {
            tracing::info!("generating");
            let query = q.query.trim_end_matches("喵");
            let model = *self.model.lock().await;
            let hint = match self.zero_client.request_new_hint(query, model).await {
                Ok(hint) => hint,
                Err(e) => {
                    tracing::error!("failed to generate hint: {e}");
                    format!("Failed to generate hint: {e}")
                }
            };

            InlineQueryResultArticle::new(
                "01".to_string(),
                "Generate",
                InputMessageContent::Text(InputMessageContentText::new(format!(
                    "Generated from {}:\n{}",
                    model.to_string(), hint,
                ))),
            )
        };

        let results = vec![InlineQueryResult::Article(msg)];

        self.bot.answer_inline_query(&q.id, results).send().await?;
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
        let model = *self.model.lock().await;

        let hint = match self.zero_client.request_new_hint(prompt, model).await {
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

    async fn handle_model(&self, msg: Message, input: String) -> anyhow::Result<()> {
        let model = Model::from(input.as_str());
        if model == Model::Invalid {
            self.bot.send_message(msg.chat.id, "Invalid model").await?;

            return Ok(());
        }

        *self.model.lock().await = model;
        self.bot
            .send_message(msg.chat.id, format!("Model changed to {model}"))
            .await?;
        tracing::info!("model changed to {model}");
        Ok(())
    }

    async fn handle_look_model(&self, msg: Message) -> anyhow::Result<()> {
        let model = self.model.lock().await;
        let str = model.to_string();
        self.bot
            .send_message(msg.chat.id, format!("Current model: {str}"))
            .await?;
        tracing::info!("look at the current model");
        Ok(())
    }

    pub async fn run_active(&self) -> anyhow::Result<()> {
        let handler = dptree::entry()
            .branch(Update::filter_inline_query().endpoint({
                let bot = self.clone();

                move |_: teloxide::Bot, q: InlineQuery| {
                    let bot = bot.clone();

                    async move { bot.handle_inline(q).await }
                }
            }))
            .branch(Update::filter_message().endpoint({
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
