use log;
use pretty_env_logger;
use std::sync::Arc;
use std::{error::Error, sync::Mutex};
use teloxide::{prelude::*, utils::command::BotCommands};

type GameState = Arc<Mutex<Option<Game>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting That Mafia Game Bot");

    let players = Arc::new(Mutex::new(Option::<Game>::None));

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<StartCommand>()
                .endpoint(start_command_handler),
        )
        .branch(
            dptree::filter(|gs: GameState, msg: Message| {
                if let Some(g) = gs.lock().unwrap().as_ref() {
                    return g.players.iter().any(|p| p.id == msg.chat.id);
                } else {
                    false
                }
            })
            .endpoint(|gs: GameState, bot: Bot, msg: Message| async move {
                bot.send_message(
                    msg.chat.id,
                    format!(
                        "Hello, you are in #{}",
                        gs.lock().unwrap().as_ref().unwrap().code
                    ),
                )
                .await?;

                Ok(())
            }),
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
    game: GameState,
    bot: Bot,
    msg: Message,
    cmd: StartCommand,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        StartCommand::Help => StartCommand::descriptions().to_string(),
        StartCommand::Host => {
            let mut g = game.lock().unwrap();
            match g.as_ref() {
                Some(g) => {
                    format!("Existing game room: {}", g.code)
                }
                None => {
                    let _ = g.insert(Game {
                        host: msg.chat.id,
                        players: vec![Player {
                            id: msg.chat.id,
                            username: String::from(msg.chat.username().unwrap()),
                        }],
                        code: 123,
                    });
                    format!("Creating new game: code={}", 123)
                }
            }
        }
        StartCommand::Join { code: c } => {
            let mut g = game.lock().unwrap();
            match g.as_mut() {
                Some(Game {
                    code: asdf,
                    players: p,
                    ..
                }) if asdf.to_owned() == c => {
                    p.push(Player {
                        id: msg.chat.id,
                        username: String::from(msg.chat.username().unwrap()),
                    });
                    format!("Joining game #{}", asdf)
                }
                _ => {
                    format!("Cannot find game #{}", c)
                }
            }
        }
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
