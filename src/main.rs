use log;
use pretty_env_logger;
use teloxide::macros::BotCommands;
use std::sync::Arc;
use std::{error::Error, sync::Mutex};
use teloxide::{prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting That Mafia Game Bot");

    let players = Arc::new(Mutex::new(Option::<Game>::None));

    let handler = Update::filter_message().endpoint(msg_handler);

    let bot = Bot::from_env();
    Dispatcher::builder(
        bot,
        Update::filter_message().endpoint(msg_handler),
    )
    .dependencies(dptree::deps![players])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;

    Ok(())
}
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Start commands")]
enum StartCommand {
    #[command(description = "shows this message.")]
    Help,
    #[command(description = "host a game")]
    Host,
    #[command(parse_with = "split", description = "join a game")]
    Join,
}

struct Player {
    id: ChatId,
    username: String,
}

struct Game {
    host: ChatId,
    players: Vec<Player>,
    code: i32,
}

async fn msg_handler(
    bot: Bot,
    players: Arc<Mutex<Vec<String>>>,
    msg: Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    players
        .lock()
        .unwrap()
        .push(String::from(msg.chat.username().unwrap()));
    bot.send_message(msg.chat.id, format!("{:?}", players.lock().unwrap()))
        .await?;
    Ok(())
}
