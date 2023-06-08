mod local_lobby_manager;

use teloxide::prelude::*;

#[derive(Eq, Hash, PartialEq, Copy, Clone)]
pub struct LobbyId(pub i32);

// Provides a snapshot of a lobby's details
pub struct Lobby {
    host: ChatId,
    players: Vec<ChatId>,
    lobby_id: LobbyId,
}

pub trait LobbyManager {
    // Gets the instantaneous lobby, if present, of a chat user.
    fn get_chats_lobby(&mut self, chat_id: ChatId) -> Option<&Lobby>;

    fn create_lobby(&mut self, host_chat_id: ChatId) -> Result<&Lobby, &'static str>;

    fn join_lobby(&mut self, lobby_id: LobbyId, chat_id: ChatId) -> Result<&Lobby, &'static str>;

    // If the host quits, then a remaining player should be randomly chosen to be the new host
    fn quit_lobby(&mut self, chat_id: ChatId) -> Result<(), &'static str>;
}
