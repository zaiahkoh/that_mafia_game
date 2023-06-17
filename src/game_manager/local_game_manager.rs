use super::{Game, GameId, GameManager};
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
    fn get_player_game(&self, chat_id: ChatId) -> Option<&Game> {
        let game_id = self.player_map.get(&chat_id)?;
        self.games.get(game_id)
    }

    fn add_game(&mut self, game: Game) {
        let mut rng = rand::thread_rng();
        let mut game_id = GameId(rng.gen_range(1_000..10_000));
        while let Some(_) = self.games.get(&game_id) {
            game_id = GameId(rng.gen_range(1_000..10_000));
        }

        for p in game.players.iter() {
            self.player_map.insert(p.player_id, game_id);   
        }

        self.games.insert(game_id, game);
    }

    fn update_game(&mut self, game: Game, chat_id: ChatId) {
        let game_id = self.player_map.get(&chat_id);
        self.games.insert(*game_id.unwrap(), game);
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
