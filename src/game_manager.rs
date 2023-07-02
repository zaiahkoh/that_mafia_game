use std::collections::HashMap;

use rand::{seq::SliceRandom, thread_rng};
use teloxide::types::{ChatId, MessageId};

use crate::lobby_manager::Lobby;

#[derive(Copy, Clone, Debug)]
pub enum Role {
    Mafia,
    Civilian,
}

#[derive(Clone)]
pub struct Player {
    pub chat_id: ChatId,
    pub username: String,
    pub role: Role,
    pub is_alive: bool,
    pub is_connected: bool,
}

#[derive(Clone)]
pub struct Game {
    pub players: Vec<Player>,
    pub phase: GamePhase,
    pub previous: Option<Box<Game>>,
}

#[derive(Clone)]
pub enum GamePhase {
    Night {
        count: i32,
        actions: Vec<Action>,
    },
    Voting {
        count: i32,
        poll_id_map: HashMap<ChatId, MessageId>,
        vote_options: Vec<(ChatId, String)>,
        votes: HashMap<ChatId, Vec<ChatId>>,
        prev_votes: Option<HashMap<ChatId, Vec<ChatId>>>,
    },
    Trial {
        count: i32,
    },
}

pub const VOTE_OPTION_NOBODY: ChatId = ChatId(-1);
pub const VOTE_OPTION_ABSTAIN: ChatId = ChatId(-2);

impl Game {
    pub fn from_lobby(lobby: &Lobby) -> Game {
        let player_count = lobby.players.len();
        let mut roles = vec![Role::Civilian; player_count];
        roles[0] = Role::Mafia;
        roles.shuffle(&mut thread_rng());

        let players = lobby
            .players
            .iter()
            .zip(roles)
            .map(|(p, r)| Player {
                chat_id: p.player_id,
                username: p.username.clone(),
                is_alive: true,
                role: r,
                is_connected: true,
            })
            .collect::<Vec<_>>();

        Game {
            players,
            phase: GamePhase::Night {
                count: 0,
                actions: vec![],
            },
            previous: None,
        }
    }

    pub fn get_winner(&self) -> Option<String> {
        let mafia_count = self
            .players
            .iter()
            .filter(|p| matches!(p.role, Role::Mafia))
            .count();
        let civilian_count = self
            .players
            .iter()
            .filter(|p| !matches!(p.role, Role::Civilian))
            .count();
        if mafia_count == 0 {
            Some(String::from("Civilians"))
        } else if mafia_count >= civilian_count {
            Some(String::from("Mafia"))
        } else {
            None
        }
    }

    pub fn get_player(&self, chat_id: ChatId) -> Option<&Player> {
        self.players.iter().find(|p| p.chat_id == chat_id)
    }

    pub fn get_alive_players(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().filter(|p| p.is_alive)
    }

    fn get_vote_targets(&self) -> impl Iterator<Item = (ChatId, String)> + '_ {
        let options = vec![
            (VOTE_OPTION_NOBODY, "Nobody".to_string()),
            (VOTE_OPTION_ABSTAIN, "Abstain".to_string()),
        ]
        .into_iter();

        self.players
            .iter()
            .filter(|p| p.is_alive)
            .map(|p| (p.chat_id, p.username.clone()))
            .chain(options)
    }

    pub fn get_role(&self, chat_id: ChatId) -> Option<Role> {
        self.players
            .iter()
            .find(|p| p.chat_id == chat_id)?
            .role
            .into()
    }

    pub fn push_night_action(&mut self, action: Action) -> Result<(), &'static str> {
        if let GamePhase::Night { actions, .. } = &mut self.phase {
            actions.push(action);
            Ok(())
        } else {
            Err("Internal error: night_action called when not GamePhase::Night")
        }
    }

    pub fn count_night_pending_players(&self) -> Result<usize, &'static str> {
        if let GamePhase::Night { actions, .. } = &self.phase {
            let is_player_idle = |p: &&Player| match p.role {
                Role::Mafia => actions
                    .iter()
                    .find(|a| match a {
                        Action::Kill { source, .. } if source == &p.chat_id => true,
                        _ => false,
                    })
                    .is_none(),
                Role::Civilian => false,
            };

            let idle_player_count = self.players.iter().filter(is_player_idle).count();

            Ok(idle_player_count)
        } else {
            Err("Internal error: is_night_done called when not GamePhase::Night")
        }
    }

    pub fn end_night(&mut self) -> Result<(), &'static str> {
        if let GamePhase::Night { actions, count } = &self.phase {
            self.previous = Some(Box::new(self.clone()));
            // Resolve actions
            for a in actions {
                match a {
                    Action::Kill { target, .. } => {
                        if let Some(target) = self.players.iter_mut().find(|p| p.chat_id == *target)
                        {
                            target.is_alive = false;
                        }
                    }
                }
            }

            self.phase = GamePhase::Voting {
                count: *count,
                votes: HashMap::new(),
                prev_votes: None,
                poll_id_map: HashMap::new(),
                vote_options: self.get_vote_targets().collect::<Vec<_>>(),
            };
            Ok(())
        } else {
            Err("Internal error: is_night_done called when not GamePhase::Night")
        }
    }

    pub fn count_voting_pending_players(&self) -> Result<usize, &'static str> {
        if let GamePhase::Voting { .. } = &self.phase {
            let idle_player_count = self.players.iter().filter(|p| p.is_alive).count();
            Ok(idle_player_count)
        } else {
            Err("Internal error: is_night_done called when not GamePhase::Night")
        }
    }

    pub fn get_transition_message(&self) -> String {
        match &self.phase {
            GamePhase::Night { count, actions } => {
                if let Some(Game {
                    players,
                    phase: GamePhase::Trial { .. },
                    ..
                }) = self.previous.as_deref()
                {
                    "Not implemented yet".to_string()
                } else {
                    panic!("get_transition_message: game.previous.phase does not match")
                }
            }
            GamePhase::Voting { count, .. } => {
                if let Some(Game {
                    players,
                    phase: GamePhase::Night { .. },
                    ..
                }) = self.previous.as_deref()
                {
                    let killed_player_names = players
                        .iter()
                        .filter(|p| p.is_alive && !self.get_player(p.chat_id).unwrap().is_alive)
                        .map(|p| p.username.clone())
                        .collect::<Vec<_>>()
                        .join(", ");

                    format!("{killed_player_names} were killed last night!")
                } else {
                    panic!("get_transition_message: game.previous.phase does not match")
                }
            }

            GamePhase::Trial { count } => "Not implemented yet".to_string(),
        }
    }

    pub fn add_poll_id_map(&mut self, pim: HashMap<ChatId, MessageId>) -> Result<(), &'static str> {
        if let GamePhase::Voting { poll_id_map, .. } = &mut self.phase {
            pim.clone_into(poll_id_map);

            Ok(())
        } else {
            Err("add_poll_ids called when not in GamePhase::Voting")
        }
    }

    pub fn add_votes(
        &mut self,
        voter_id: ChatId,
        chosen: Vec<i32>,
    ) -> Result<Vec<String>, &'static str> {
        if let GamePhase::Voting {
            votes,
            vote_options,
            ..
        } = &mut self.phase
        {
            let target_ids = chosen
                .iter()
                .map(|idx| vote_options[*idx as usize].0)
                .collect::<Vec<_>>();
            votes.insert(voter_id, target_ids);

            let target_usernames = chosen
                .iter()
                .map(|idx| vote_options[*idx as usize].1.clone())
                .collect::<Vec<_>>();

            Ok(target_usernames)
        } else {
            Err("add_poll_ids called when not in GamePhase::Voting")
        }
    }

    pub fn get_voter_poll_msg_id(&self, voter_id: ChatId) -> Result<MessageId, &'static str> {
        if let GamePhase::Voting { poll_id_map, .. } = &self.phase {
            Ok(poll_id_map[&voter_id])
        } else {
            Err("get_voter_poll_msg_id called when not in GamePhase::Voting")
        }
    }

    pub fn get_vote_options(&self) -> Result<Vec<(ChatId, String)>, &'static str> {
        if let GamePhase::Voting { vote_options, .. } = &self.phase {
            Ok(vote_options.clone())
        } else {
            Err("get_voter_poll_msg_id called when not in GamePhase::Voting")
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
    fn get_player_game(&mut self, chat_id: ChatId) -> Option<&mut Game>;

    // Adds game to the map
    fn add_game(&mut self, game: Game);

    fn update_game(&mut self, game: Game, chat_id: ChatId);

    // If the host quits, then a remaining player should be randomly chosen to be the new host
    fn quit_game(&mut self, chat_id: ChatId) -> Result<&Game, &'static str>;
}

pub mod local_game_manager;

//game_manager keeps track of games progress and player roles (data)
//game_handler handles replies and prompts. Also decides which prompts to give out (logic)
