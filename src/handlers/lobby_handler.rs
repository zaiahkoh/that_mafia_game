use teloxide::{prelude::*, utils::command::BotCommands};

use super::{AsyncBotState, game_handler::start_night};
use crate::{lobby_manager::LobbyManager, game_manager::Game};

pub fn get_lobby_handler() -> Handler<
    'static,
    DependencyMap,
    Result<(), teloxide::RequestError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    dptree::filter(|msg: Message, bot_state: AsyncBotState| {
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
    let mut o_game: Option<Game> = None;
    let text = match cmd {
        LobbyCommand::Help => LobbyCommand::descriptions().to_string(),
        LobbyCommand::Players => {
            let mut state_lock = bot_state.lock().unwrap();
            match state_lock.lobby_manager.get_chats_lobby(msg.chat.id) {
                Some(lobby) => {
                    let host_id = lobby.host;
                    let mut player_index = 0;
                    lobby
                        .players
                        .iter()
                        .map(|p| -> String {
                            player_index += 1;
                            if p.player_id == host_id {
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
            // let mut state_lock = bot_state.lock().unwrap();
            let lobby_manager = &mut bot_state.lock().unwrap().lobby_manager;
            if let Some(lobby) = lobby_manager.get_chats_lobby(msg.chat.id) {
                let game = Game::from_lobby(lobby);
                lobby_manager.close_lobby(lobby.lobby_id);
                o_game = Some(game);
            }

            format!("Started lobby")
        }
    };

    bot.send_message(msg.chat.id, text).await?;
    if let Some(game) = o_game {
        start_night(&game, bot).await;
    }

    Ok(())
}