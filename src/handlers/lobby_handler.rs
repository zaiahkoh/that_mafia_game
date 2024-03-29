use teloxide::{prelude::*, utils::command::BotCommands};

use super::{game_handler::start_night, AsyncBotState};
use crate::game::{game_v1::GameV1, Game};
use crate::{game_manager::GameManager, lobby_manager::LobbyManager};

pub fn get_lobby_handler() -> Handler<
    'static,
    DependencyMap,
    Result<(), teloxide::RequestError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    Update::filter_message()
        .filter(|msg: Message, bot_state: AsyncBotState| {
            bot_state
                .lock()
                .unwrap()
                .lobby_manager
                .get_chats_lobby(msg.chat.id)
                .is_some()
        })
        .filter_command::<LobbyCommand>()
        .endpoint(lobby_handler)
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Lobby commands")]
enum LobbyCommand {
    #[command(description = "Shows the message.")]
    Help,
    #[command(description = "List players in the lobby")]
    Players,
    #[command(description = "Quit lobby")]
    Quit,
    #[command(description = "Start game")]
    Start,
}

async fn lobby_handler(
    bot_state: AsyncBotState,
    bot: Bot,
    msg: Message,
    cmd: LobbyCommand,
) -> Result<(), teloxide::RequestError> {
    let mut game_opt: Option<Box<dyn Game>> = None;
    let text = match cmd {
        LobbyCommand::Help => LobbyCommand::descriptions().to_string(),
        LobbyCommand::Players => {
            let state_lock = bot_state.lock().unwrap();
            match state_lock.lobby_manager.get_chats_lobby(msg.chat.id) {
                Some(lobby) => {
                    let host_id = lobby.host_id;
                    let mut player_index = 0;
                    lobby
                        .users
                        .iter()
                        .map(|p| -> String {
                            player_index += 1;
                            if p.chat_id == host_id {
                                format!("{}. {} (host)", player_index, p.username)
                            } else {
                                format!("{}. {}", player_index, p.username)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                None => format!("Internal error: player should be in a lobby but is not!"),
            }
        }
        LobbyCommand::Quit => {
            let mut state_lock = bot_state.lock().unwrap();
            state_lock
                .lobby_manager
                .get_chats_lobby(msg.chat.id)
                .unwrap();
            match state_lock.lobby_manager.quit_lobby(msg.chat.id) {
                Ok(lobby_id) => format!("Quit lobby: {}", lobby_id),
                Err(message) => format!("Encountered error: {}", message),
            }
        }
        LobbyCommand::Start => {
            let mut state_lock = bot_state.lock().unwrap();
            let lobby_manager = &mut state_lock.lobby_manager;

            if let Some(lobby) = lobby_manager.get_chats_lobby(msg.chat.id) {
                if lobby.users.len() >= 3 {
                    let game = GameV1::from_lobby(lobby);
                    if let Err(err) = lobby_manager.close_lobby(lobby.lobby_id) {
                        panic!("{err}");
                    };
                    game_opt = Some(game.snapshot());
                    state_lock.game_manager.add_game(Box::new(game));

                    format!("Started lobby")
                } else {
                    format!("Cannot start game: Need 3 or more players")
                }
            } else {
                format!("Internal error: failed to find lobby to start")
            }
        }
    };

    bot.send_message(msg.chat.id, text).await?;
    if let Some(_) = game_opt {
        start_night(msg.chat.id, bot, bot_state).await;
    }

    Ok(())
}
