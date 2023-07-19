use super::GameManager;
use crate::game_interface::Game;
use rand::Rng;
use std::collections::HashMap;
use teloxide::types::ChatId;
// use crate::game_interface::game_v1::GameV1;

#[derive(Eq, Hash, PartialEq, Copy, Clone, derive_more::Display)]
pub struct GameId(pub i32);

pub struct LocalGameManager<G>
where
    G: Game,
{
    games: HashMap<GameId, G>,
    player_map: HashMap<ChatId, GameId>,
}

impl<G: Game> LocalGameManager<G> {
    pub fn new() -> LocalGameManager<G> {
        LocalGameManager {
            games: HashMap::new(),
            player_map: HashMap::new(),
        }
    }
}

impl<G: Game> GameManager<G> for LocalGameManager<G> {
    fn get_player_game(&mut self, chat_id: ChatId) -> Option<&mut G> {
        let game_id = self.player_map.get(&chat_id)?;
        self.games.get_mut(game_id)
    }

    fn add_game(&mut self, game: G)
    where
        G: Game,
    {
        let mut rng = rand::thread_rng();
        let mut game_id = GameId(rng.gen_range(1_000..10_000));
        while let Some(_) = self.games.get(&game_id) {
            game_id = GameId(rng.gen_range(1_000..10_000));
        }

        for p in game.players.iter() {
            self.player_map.insert(p.chat_id, game_id);
        }

        self.games.insert(game_id, game);
    }

    fn update_game(&mut self, game: G, chat_id: ChatId) {
        let game_id = self.player_map.get(&chat_id);
        self.games.insert(*game_id.unwrap(), game);
    }

    fn quit_game(&mut self, chat_id: ChatId) -> Result<&mut G, &'static str> {
        match self.player_map.get(&chat_id) {
            Some(game_id) => {
                let game = self.games.get_mut(game_id).unwrap();
                let player = game
                    .players
                    .iter_mut()
                    .find(|p| p.chat_id == chat_id)
                    .unwrap();
                player.is_connected = false;

                self.player_map.remove(&chat_id);
                return Ok(game);
            }
            None => Err("Player not in a game"),
        }
    }
}
