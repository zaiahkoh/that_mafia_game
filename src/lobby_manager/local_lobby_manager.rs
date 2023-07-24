use rand::Rng;
use std::collections::HashMap;

use crate::lobby_manager::*;

pub struct LocalLobbyManager {
    lobbies: HashMap<LobbyId, Lobby>,
    user_map: HashMap<ChatId, LobbyId>,
}

impl LocalLobbyManager {
    pub fn new() -> LocalLobbyManager {
        LocalLobbyManager {
            lobbies: HashMap::new(),
            user_map: HashMap::new(),
        }
    }
}

impl LobbyManager for LocalLobbyManager {
    fn get_chats_lobby(&self, chat_id: teloxide::types::ChatId) -> Option<&Lobby> {
        let lobby_id = self.user_map.get(&chat_id)?;
        self.lobbies.get(lobby_id)
    }

    fn create_lobby(&mut self, user: User) -> Result<&Lobby, &'static str> {
        if let Some(_) = self.get_chats_lobby(user.chat_id) {
            return Err("User is already in a lobby");
        }

        let mut rng = rand::thread_rng();
        let mut lobby_id = LobbyId(rng.gen_range(1_000..10_000));
        while let Some(_) = self.lobbies.get(&lobby_id) {
            lobby_id = LobbyId(rng.gen_range(1_000..10_000));
        }
        let chat_id = user.chat_id.clone();

        let lobby = Lobby {
            host_id: user.chat_id,
            users: vec![user],
            lobby_id,
        };

        self.lobbies.insert(lobby_id, lobby);
        self.user_map.insert(chat_id, lobby_id);

        return Ok(self.lobbies.get(&lobby_id).unwrap());
    }

    fn join_lobby(&mut self, lobby_id: LobbyId, user: User) -> Result<&Lobby, &'static str> {
        if let Some(_) = self.get_chats_lobby(user.chat_id) {
            return Err("User is already in a lobby");
        }

        let chat_id = user.chat_id;

        match self.lobbies.get_mut(&lobby_id) {
            Some(lobby) => {
                lobby.users.push(user);
                self.user_map.insert(chat_id, lobby.lobby_id);
                Ok(lobby)
            }
            None => Err("Lobby does not exist"),
        }
    }

    fn close_lobby(&mut self, lobby_id: LobbyId) -> Result<(), &'static str> {
        let users = self.lobbies.get(&lobby_id).unwrap().users.iter();
        for p in users {
            self.user_map.remove(&p.chat_id);
        }
        self.lobbies.remove(&lobby_id);

        Ok(())
    }

    fn quit_lobby(&mut self, chat_id: ChatId) -> Result<LobbyId, &'static str> {
        if let Some(lobby_id) = self.user_map.get(&chat_id) {
            if let Some(lobby) = self.lobbies.get_mut(&lobby_id) {
                if lobby.users.len() == 1 {
                    self.lobbies.remove(lobby_id);
                } else if lobby.host_id == chat_id {
                    lobby.users.retain(|p| p.chat_id != chat_id);
                    lobby.host_id = lobby.users[0].chat_id;
                }

                let ret = lobby_id.clone();
                self.user_map.remove(&chat_id);

                Ok(ret)
            } else {
                Err("Internal error: user_map and lobbies not synced")
            }
        } else {
            Err("Chat ID is not in any lobby")
        }
    }
}
