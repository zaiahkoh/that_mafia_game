use super::GameManager;
use crate::game_interface::Game;
use rand::Rng;
use std::collections::HashMap;
use teloxide::types::ChatId;
// use crate::game_interface::game_v1::GameV1;

#[derive(Eq, Hash, PartialEq, Copy, Clone, derive_more::Display)]
pub struct GameId(pub i32);

pub struct LocalGameManager {
    games: HashMap<GameId, Box<dyn Game>>,
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
    fn get_player_game(&mut self, chat_id: ChatId) -> Option<&mut dyn Game> {
        let game_id = self.player_map.get(&chat_id)?;
        self.games.get_mut(game_id).map(|g| g.as_mut())
    }

    fn add_game(&mut self, game: Box<dyn Game>) {
        let mut rng = rand::thread_rng();
        let mut game_id = GameId(rng.gen_range(1_000..10_000));
        while let Some(_) = self.games.get(&game_id) {
            game_id = GameId(rng.gen_range(1_000..10_000));
        }

        for p in game.get_players() {
            self.player_map.insert(p.chat_id, game_id);
        }

        self.games.insert(game_id, game);
    }

    fn quit_game(&mut self, chat_id: ChatId) -> Result<&mut dyn Game, &'static str> {
        match self.player_map.get(&chat_id) {
            Some(game_id) => {
                let game = self.games.get_mut(game_id).unwrap();

                self.player_map.remove(&chat_id);
                return Ok(game.as_mut());
            }
            None => Err("Player not in a game"),
        }
    }
}
