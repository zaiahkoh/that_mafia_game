use super::{Game, GameId, GameManager, Player, Role};
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

    fn get_player_role(&self, chat_id: ChatId) -> Result<Role, &'static str> {
        match self.get_player_game(chat_id) {
            Some(game) => match game.players.iter().find(|p| p.player_id == chat_id) {
                Some(player) => Ok(player.role),
                None => Err("Player is not in a game"),
            },
            None => Err("Player is not in a game"),
        }
    }

    fn from_lobby(&mut self, lobby: crate::lobby_manager::Lobby) -> Result<&Game, &'static str> {
        let mut rng = rand::thread_rng();
        let mut game_id = GameId(rng.gen_range(1_000..10_000));
        while let Some(_) = self.games.get(&game_id) {
            game_id = GameId(rng.gen_range(1_000..10_000));
        }

        let game = Game {
            game_id,
            players: lobby
                .players
                .iter()
                .map(|p| Player {
                    player_id: p.player_id,
                    username: p.username.clone(),
                    is_alive: true,
                    role: Role::Civilian,
                    is_connected: true,
                })
                .collect::<Vec<_>>(),
            day: 0,
        };

        for p in lobby.players {
            self.player_map.insert(p.player_id, game_id);
        }
        self.games.insert(game_id, game);

        return Ok(self.games.get(&game_id).unwrap());
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
