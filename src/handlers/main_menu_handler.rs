use teloxide::{prelude::*, utils::command::BotCommands};

use super::AsyncBotState;
use crate::lobby_manager::*;

pub fn get_main_menu_handler() -> Handler<
    'static,
    DependencyMap,
    Result<(), teloxide::RequestError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    dptree::entry()
        .filter(|msg: Message, bot_state: AsyncBotState| {
            bot_state
                .lock()
                .unwrap()
                .lobby_manager
                .get_chats_lobby(msg.chat.id)
                .is_none()
        })
        .filter_command::<MainMenuCommand>()
        .endpoint(main_menu_handler)
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Main menu commands")]
enum MainMenuCommand {
    #[command(description = "Shows this message.")]
    Help,
    #[command(description = "Host a lobby")]
    Host,
    #[command(description = "Join a lobby")]
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

            match state_lock.lobby_manager.create_lobby(Player {
                player_id: msg.chat.id,
                username: String::from(msg.chat.username().unwrap_or("(no name)")),
            }) {
                Ok(lobby) => {
                    format!("Created new lobby. Code: {}", lobby.lobby_id)
                }
                Err(message) => format!("Encountered error: {}", message),
            }
        }
        MainMenuCommand::Join { code } => {
            let mut state_lock = bot_state.lock().unwrap();

            match state_lock.lobby_manager.join_lobby(
                LobbyId(code),
                Player {
                    player_id: msg.chat.id,
                    username: String::from(msg.chat.username().unwrap_or("(no name)")),
                },
            ) {
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
