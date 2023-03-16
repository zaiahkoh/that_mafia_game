use log;
use pretty_env_logger;
use std::sync::Arc;
use std::{error::Error, sync::Mutex};
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting That Mafia Game Bot");

    let players = Arc::new(Mutex::new(Option::<Game>::None));

    let handler = Update::filter_message().branch(
        dptree::entry()
            .filter_command::<StartCommand>()
            .endpoint(start_command_handler),
    );

    let bot = Bot::from_env();
    Dispatcher::builder(bot, handler)
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
    #[command(description = "join a game")]
    Join { code: i32 },
}

async fn start_command_handler(
    game: Arc<Mutex<Option<Game>>>,
    bot: Bot,
    msg: Message,
    cmd: StartCommand,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        StartCommand::Help => StartCommand::descriptions().to_string(),
        _ => String::from("not recognised"),
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
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
