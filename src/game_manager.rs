use teloxide::types::ChatId;

use crate::lobby_manager::Lobby;

#[derive(Copy, Clone)]
pub enum Role {
    Mafia,
    Civilian,
}

#[derive(Clone)]
pub struct Player {
    pub player_id: ChatId,
    pub username: String,
    pub role: Role,
    pub is_alive: bool,
    pub is_connected: bool,
}

#[derive(Eq, Hash, PartialEq, Copy, Clone, derive_more::Display)]
pub struct GameId(pub i32);

#[derive(Clone)]
pub struct Game {
    pub players: Vec<Player>,
    pub phase: GamePhase,
}

#[derive(Clone)]
pub enum GamePhase {
    Night { count: i32, actions: Vec<Action> },
    Voting { count: i32 },
    Trial { count: i32 },
}

impl Game {
    pub fn from_lobby(lobby: &Lobby) -> Game {
        Game {
            players: lobby
                .players
                .iter()
                .map(|p| Player {
                    player_id: p.player_id,
                    username: p.username.clone(),
                    is_alive: true,
                    role: Role::Mafia,
                    is_connected: true,
                })
                .collect::<Vec<_>>(),
            phase: GamePhase::Night {
                count: 0,
                actions: vec![],
            },
        }
    }

    pub fn get_player(&self, chat_id: ChatId) -> Option<&Player> {
        self.players.iter().find(|p| p.player_id == chat_id)
    }

    pub fn get_role(&self, chat_id: ChatId) -> Option<Role> {
        self.players
            .iter()
            .find(|p| p.player_id == chat_id)?
            .role
            .into()
    }

    pub fn night_action(&mut self, action: Action) -> Result<(), &'static str> {
        if let GamePhase::Night { actions, .. } = &mut self.phase {
            actions.push(action);
            Ok(())
        } else {
            Err("Internal error: night_action called when not GamePhase::Night")
        }
    }

    pub fn count_night_pending_players(&self) -> Result<usize, &'static str> {
        if let GamePhase::Night { actions, .. } = &self.phase {
            let idle_player_count = self
                .players
                .iter()
                .filter(|p| match p.role {
                    Role::Mafia => actions
                        .iter()
                        .find(|a| match a {
                            Action::Kill { source, .. } if source == &p.player_id => true,
                            _ => false,
                        })
                        .is_none(),
                    Role::Civilian => false,
                })
                .count();

            Ok(idle_player_count)
        } else {
            Err("Internal error: is_night_done called when not GamePhase::Night")
        }
    }
}

#[derive(Clone)]
pub enum Action {
    Kill { source: ChatId, target: ChatId },
}

impl GamePhase {}

pub trait GameManager {
    // Gets the instantaneous lobby, if present, of a chat user.
    fn get_player_game(&self, chat_id: ChatId) -> Option<&Game>;

    // Adds game to the map
    fn add_game(&mut self, game: Game);

    fn update_game(&mut self, game: Game, chat_id: ChatId);

    // If the host quits, then a remaining player should be randomly chosen to be the new host
    fn quit_game(&mut self, chat_id: ChatId) -> Result<&Game, &'static str>;
}

pub mod local_game_manager;

//game_manager keeps track of games progress and player roles (data)
//game_handler handles replies and prompts. Also decides which prompts to give out (logic)
