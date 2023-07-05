use std::collections::HashMap;

use teloxide::types::ChatId;

use crate::lobby_manager::Lobby;

pub struct Player {
    chat_id: ChatId,
    username: String,
}

pub trait Game {
    fn from_lobby(lobby: &Lobby) -> Self;

    fn get_night_targets(&self) -> HashMap<ChatId, Vec<&Player>>;

    fn add_action(&mut self, action: Action) -> Result<(), &'static str>;

    fn end_phase(&mut self);
}
