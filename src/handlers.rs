use std::sync::{Arc, Mutex};

use crate::{
    game_manager::{local_game_manager::LocalGameManager, GameManager},
    lobby_manager::{local_lobby_manager::LocalLobbyManager, LobbyManager},
};

pub struct BotState<L: LobbyManager, G: GameManager> {
    pub lobby_manager: L,
    pub game_manager: G,
}

pub type AsyncBotState = Arc<Mutex<BotState<LocalLobbyManager, LocalGameManager>>>;

pub fn new_async_bot_state() -> AsyncBotState {
    Arc::new(Mutex::new(BotState {
        lobby_manager: LocalLobbyManager::new(),
        game_manager: LocalGameManager::new(),
    }))
}

pub mod game_handler;
pub mod lobby_handler;
pub mod main_menu_handler;
