use std::sync::{Arc, Mutex};

use crate::lobby_manager::{local_lobby_manager::LocalLobbyManager, LobbyManager};

pub struct BotState<L: LobbyManager> {
    pub lobby_manager: L,
}

pub type AsyncBotState = Arc<Mutex<BotState<LocalLobbyManager>>>;

pub fn new_async_bot_state() -> AsyncBotState {
    Arc::new(Mutex::new(BotState {
        lobby_manager: LocalLobbyManager::new(),
    }))
}

pub mod lobby_handler;
pub mod main_menu_handler;
