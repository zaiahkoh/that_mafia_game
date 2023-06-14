use std::collections::HashMap;

use crate::lobby_manager::*;
use rand::Rng;

pub struct LocalLobbyManager {
    lobbies: HashMap<LobbyId, Lobby>,
    player_map: HashMap<ChatId, LobbyId>,
}

impl LocalLobbyManager {
    pub fn new() -> LocalLobbyManager {
        LocalLobbyManager {
            lobbies: HashMap::new(),
            player_map: HashMap::new(),
        }
    }
}

impl LobbyManager for LocalLobbyManager {
    fn get_chats_lobby(&mut self, chat_id: teloxide::types::ChatId) -> Option<&Lobby> {
        let lobby_id = self.player_map.get(&chat_id)?;
        return self.lobbies.get(lobby_id);
    }

    fn create_lobby(&mut self, player: Player) -> Result<&Lobby, &'static str> {
        if let Some(_) = self.get_chats_lobby(player.player_id) {
            return Err("Player is already in a lobby");
        }

        let mut rng = rand::thread_rng();
        let mut lobby_id = LobbyId(rng.gen_range(1_000..10_000));
        while let Some(_) = self.lobbies.get(&lobby_id) {
            lobby_id = LobbyId(rng.gen_range(1_000..10_000));
        }
        let player_id = player.player_id.clone();

        let lobby = Lobby {
            host: player.player_id,
            players: vec![player],
            lobby_id,
        };

        self.lobbies.insert(lobby_id, lobby);
        self.player_map.insert(player_id, lobby_id);

        return Ok(self.lobbies.get(&lobby_id).unwrap());
    }

    fn join_lobby(&mut self, lobby_id: LobbyId, player: Player) -> Result<&Lobby, &'static str> {
        if let Some(_) = self.get_chats_lobby(player.player_id) {
            return Err("Player is already in a lobby");
        }

        let player_id = player.player_id;

        match self.lobbies.get_mut(&lobby_id) {
            Some(lobby) => {
                lobby.players.push(player);
                self.player_map.insert(player_id, lobby.lobby_id);
                Ok(lobby)
            }
            None => Err("Lobby does not exist"),
        }
    }

    fn quit_lobby(&mut self, chat_id: ChatId) -> Result<LobbyId, &'static str> {
        if let Some(lobby_id) = self.player_map.get(&chat_id) {
            if let Some(lobby) = self.lobbies.get_mut(&lobby_id) {
                if lobby.players.len() == 1 {
                    self.lobbies.remove(lobby_id);
                } else if lobby.host == chat_id {
                    lobby.players.retain(|p| p.player_id != chat_id);
                    lobby.host = lobby.players[0].player_id;
                }

                let ret = lobby_id.clone();
                self.player_map.remove(&chat_id);

                Ok(ret)
            } else {
                Err("Internal error: player_map and lobbies not synced")
            }
        } else {
            Err("Chat ID is not in any lobby")
        }
    }
}
