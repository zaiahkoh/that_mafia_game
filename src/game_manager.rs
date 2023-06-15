use teloxide::types::ChatId;

use crate::lobby_manager::Lobby;

#[derive(Copy, Clone)]
pub enum Role {
    Mafia,
    Jester,
    Civilian,
}

pub struct Player {
    pub player_id: ChatId,
    pub username: String,
    pub role: Role,
    pub is_alive: bool,
    pub is_connected: bool,
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, derive_more::Display)]
pub struct GameId(pub i32);

pub struct Game {
    pub game_id: GameId,
    pub players: Vec<Player>,
    pub day: i32,
}

pub trait GameManager {
    // Gets the instantaneous lobby, if present, of a chat user.
    fn get_player_game(&self, chat_id: ChatId) -> Option<&Game>;

    fn get_player_role(&self, chat_id: ChatId) -> Result<Role, &'static str>;

    fn from_lobby(&mut self, lobby: Lobby) -> Result<&Game, &'static str>;

    // If the host quits, then a remaining player should be randomly chosen to be the new host
    fn quit_game(&mut self, chat_id: ChatId) -> Result<&Game, &'static str>;
}

pub mod local_game_manager;

//game_manager keeps track of games progress and player roles (data)
//game_handler handles replies and prompts. Also decides which prompts to give out (logic)
