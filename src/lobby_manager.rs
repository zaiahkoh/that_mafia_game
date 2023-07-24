use teloxide::prelude::*;

pub mod local_lobby_manager;

#[derive(Eq, Hash, PartialEq, Copy, Clone, derive_more::Display)]
pub struct LobbyId(pub i32);

pub struct User {
    pub chat_id: ChatId,
    pub username: String,
}

// Provides a snapshot of a lobby's details
pub struct Lobby {
    pub host_id: ChatId,
    pub users: Vec<User>,
    pub lobby_id: LobbyId,
}

pub trait LobbyManager {
    // Gets the instantaneous lobby, if present, of a chat user.
    fn get_chats_lobby(&self, chat_id: ChatId) -> Option<&Lobby>;

    fn create_lobby(&mut self, user: User) -> Result<&Lobby, &'static str>;

    fn join_lobby(&mut self, lobby_id: LobbyId, user: User) -> Result<&Lobby, &'static str>;

    fn close_lobby(&mut self, lobby_id: LobbyId) -> Result<(), &'static str>;

    // If the host quits, then a remaining player should be randomly chosen to be the new host
    fn quit_lobby(&mut self, chat_id: ChatId) -> Result<LobbyId, &'static str>;
}
