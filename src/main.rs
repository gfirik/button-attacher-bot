//! ButtonAttachBot - A Telegram bot for attaching styled buttons to posts.
//!
//! This bot allows users to send content and repost it to channels/groups
//! with customizable inline URL buttons attached.

use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::prelude::*;

mod config;
mod handlers;
mod keyboard;
mod models;
mod state;

pub use config::Config;
pub use state::BotState;

/// Type alias for the dialogue storage.
pub type Dialogue = teloxide::dispatching::dialogue::Dialogue<BotState, InMemStorage<BotState>>;

/// Type alias for handler results.
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    // Initialize logging
    pretty_env_logger::init();
    log::info!("Starting ButtonAttachBot...");

    // Load configuration
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Configuration error: {}", e);
            eprintln!("Error: {}", e);
            eprintln!("\nPlease create a .env file with your BOT_TOKEN.");
            eprintln!("See .env.example for reference.");
            std::process::exit(1);
        }
    };

    log::info!("Configuration loaded successfully");

    // Create the bot instance
    let bot = Bot::new(&config.bot_token);

    // Create dialogue storage
    let storage = InMemStorage::<BotState>::new();

    // Build the dispatcher
    let handler = build_handler();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage, config])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

/// Build the message handler tree.
fn build_handler() -> Handler<'static, DependencyMap, HandlerResult, teloxide::dispatching::DpHandlerDescription> {
    use dptree::case;
    use teloxide::dispatching::dialogue::enter;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Start].endpoint(handlers::handle_start))
        .branch(case![Command::Help].endpoint(handlers::handle_help))
        .branch(case![Command::Cancel].endpoint(handlers::handle_cancel));

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![BotState::Idle].endpoint(handlers::handle_content))
        .branch(case![BotState::AwaitingDestination { data }].endpoint(handlers::handle_destination))
        .branch(case![BotState::AwaitingButtonText { data }].endpoint(handlers::handle_button_text))
        .branch(case![BotState::AwaitingUrl { data }].endpoint(handlers::handle_url))
        .branch(case![BotState::AwaitingEmoji { data }].endpoint(handlers::handle_emoji_text));

    let callback_handler = Update::filter_callback_query()
        .branch(case![BotState::AwaitingStyle { data }].endpoint(handlers::handle_style_callback))
        .branch(case![BotState::AwaitingEmoji { data }].endpoint(handlers::handle_emoji_callback))
        .branch(case![BotState::AwaitingConfirm { data }].endpoint(handlers::handle_confirm_callback));

    enter::<Update, InMemStorage<BotState>, BotState, _>()
        .branch(message_handler)
        .branch(callback_handler)
}

/// Bot commands.
#[derive(Clone, teloxide::macros::BotCommands)]
#[command(rename_rule = "lowercase", description = "Available commands:")]
enum Command {
    #[command(description = "Show welcome message")]
    Start,
    #[command(description = "Show help information")]
    Help,
    #[command(description = "Cancel current operation")]
    Cancel,
}
