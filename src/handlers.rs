use std::sync::{Arc, Mutex};

use crate::{
    game_interface::game_v1::GameV1,
    game_manager::{local_game_manager::LocalGameManager, GameManager},
    lobby_manager::{local_lobby_manager::LocalLobbyManager, LobbyManager},
};

pub struct BotState<L: LobbyManager, G: GameManager<GameV1>> {
    pub lobby_manager: L,
    pub game_manager: G,
}

pub type AsyncBotState = Arc<Mutex<BotState<LocalLobbyManager, LocalGameManager<GameV1>>>>;

pub fn new_async_bot_state() -> AsyncBotState {
    Arc::new(Mutex::new(BotState {
        lobby_manager: LocalLobbyManager::new(),
        game_manager: LocalGameManager::new(),
    }))
}

pub mod game_handler;
pub mod lobby_handler;
pub mod main_menu_handler;
