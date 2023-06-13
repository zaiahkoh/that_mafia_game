use lobby::LobbyId;
use lobby::{local_lobby_manager::LocalLobbyManager, LobbyManager};
use std::sync::Arc;
use std::{error::Error, sync::Mutex};
use teloxide::{prelude::*, utils::command::BotCommands};

pub mod game;
pub mod lobby;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    GameRunning {
        host: ChatId,
    },
}

struct BotState<L: LobbyManager> {
    lobby_manager: L,
}

type AsyncBotState = Arc<Mutex<BotState<LocalLobbyManager>>>;

pub async fn start_mafia_bot() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting That Mafia Game Bot");

    let bot_state = Arc::new(Mutex::new(BotState {
        lobby_manager: LocalLobbyManager::new(),
    }));

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<MainMenuCommand>()
                .endpoint(main_menu_handler),
        )
        .branch(dptree::entry().filter());

    let bot = Bot::from_env();
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![bot_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn lobby_handler(
    bot_state: AsyncBotState,
    bot: Bot,
    msg: Message,
) -> Handler<'static, DependencyMap, Result<(), teloxide::RequestError>, teloxide::dispatching::DpHandlerDescription> {
    dptree::filter(|msg: Message, bot_state: Arc<Mutex<BotState<LocalLobbyManager>>>| {
        true
    })
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Main menu commands")]
enum MainMenuCommand {
    #[command(description = "Shows this message.")]
    Help,
    #[command(description = "Host a game")]
    Host,
    #[command(description = "Join a game")]
    Join { code: i32 },
}

async fn main_menu_handler(
    bot_state: AsyncBotState,
    bot: Bot,
    msg: Message,
    cmd: MainMenuCommand,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        MainMenuCommand::Help => MainMenuCommand::descriptions().to_string(),
        MainMenuCommand::Host => {
            let mut state_lock = bot_state.lock().unwrap();

            match state_lock.lobby_manager.create_lobby(msg.chat.id) {
                Ok(lobby) => {
                    format!("Created new lobby. Code: {}", lobby.lobby_id)
                }
                Err(message) => format!("Encountered error: {}", message),
            }
        }
        MainMenuCommand::Join { code } => {
            let mut state_lock = bot_state.lock().unwrap();

            match state_lock
                .lobby_manager
                .join_lobby(LobbyId(code), msg.chat.id)
            {
                Ok(_) => {
                    format!("Joined lobby {}", code)
                }
                Err(message) => format!("Encountered error: {}", message),
            }
        }
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}
