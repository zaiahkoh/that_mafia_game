use super::{Game, GameId, GameManager, Player, Role, GamePhase};
use rand::Rng;
use std::collections::HashMap;
use teloxide::types::ChatId;

pub struct LocalGameManager {
    games: HashMap<GameId, Game>,
    player_map: HashMap<ChatId, GameId>,
}

impl LocalGameManager {
    pub fn new() -> LocalGameManager {
        LocalGameManager {
            games: HashMap::new(),
            player_map: HashMap::new(),
        }
    }
}

impl GameManager for LocalGameManager {
    fn get_player_game(&self, chat_id: teloxide::types::ChatId) -> Option<&Game> {
        let game_id = self.player_map.get(&chat_id)?;
        self.games.get(game_id)
    }

    fn quit_game(&mut self, chat_id: ChatId) -> Result<&Game, &'static str> {
        match self.player_map.get(&chat_id) {
            Some(game_id) => {
                let game = self.games.get_mut(game_id).unwrap();
                let player = game
                    .players
                    .iter_mut()
                    .find(|p| p.player_id == chat_id)
                    .unwrap();
                player.is_connected = false;

                self.player_map.remove(&chat_id);
                return Ok(game);
            }
            None => Err("Player not in a game"),
        }
    }
}