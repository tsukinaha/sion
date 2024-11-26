use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start chat.")]
    Meow(String),
    #[command(
        description = "change model. available models: gpt-4o, gpt-4o-mini, o1-preview, o1-mini."
    )]
    Model(String),
    #[command(description = "look at the current model.")]
    LookModel,
}
