use std::collections::{HashMap, HashSet};

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
    previous: Option<Box<Game>>,
    transition_message: String,
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
    },
    Trial {
        count: i32,
        defendant: ChatId,
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
            transition_message: String::from("Welcome everynyan!"),
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
            let mut killed_usernames = Vec::new();
            for a in actions {
                match a {
                    Action::Kill { target, .. } => {
                        if let Some(target) = self.players.iter_mut().find(|p| p.chat_id == *target)
                        {
                            target.is_alive = false;
                            killed_usernames.push(target.username.clone());
                        }
                    }
                }
            }

            // Update state
            self.transition_message = if killed_usernames.len() > 0 {
                format!("{} died last night", killed_usernames.join(", "))
            } else {
                format!("Nobody died last night")
            };
            self.phase = GamePhase::Voting {
                count: *count,
                votes: HashMap::new(),
                poll_id_map: HashMap::new(),
                vote_options: self.get_vote_targets().collect::<Vec<_>>(),
            };
            Ok(())
        } else {
            Err("Internal error: is_night_done called when not GamePhase::Night")
        }
    }

    pub fn count_voting_pending_players(&self) -> Result<usize, &'static str> {
        if let GamePhase::Voting { votes, .. } = &self.phase {
            let idle_player_count = self
                .players
                .iter()
                .filter(|p| p.is_alive && !votes.contains_key(&p.chat_id))
                .count();
            Ok(idle_player_count)
        } else {
            Err("Internal error: is_night_done called when not GamePhase::Night")
        }
    }

    pub fn get_transition_message(&self) -> &String {
        return &self.transition_message;
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

    fn is_voting_stalemate(&self) -> bool {
        if let GamePhase::Voting { votes, .. } = &self.phase {
            if let Some(inner) = &self.previous {
                if let Game {
                    phase: GamePhase::Voting { votes: prev, .. },
                    ..
                } = &**inner
                {
                    if votes.len() != prev.len() {
                        return false;
                    }
                    for (chat_id, targets) in votes.iter() {
                        if let Some(reference) = prev.get(chat_id) {
                            let mut old_set = HashSet::new();
                            for r in reference {
                                old_set.insert(r);
                            }

                            let mut curr_set = HashSet::new();
                            for t in targets {
                                curr_set.insert(t);
                            }

                            if !curr_set.is_subset(&old_set) || !old_set.is_subset(&curr_set) {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn end_voting(&mut self) -> Result<(), &'static str> {
        if let GamePhase::Voting {
            count,
            vote_options,
            votes,
            ..
        } = &self.phase
        {
            let mut tally = HashMap::new();
            for v in vote_options {
                tally.insert(v.0, 0);
            }
            for v in votes.values() {
                for choice in v {
                    *tally.get_mut(choice).unwrap() += 1;
                }
            }

            let (top_target, top_vote_count) =
                tally.iter().max_by_key(|(_k, v)| v.clone()).unwrap();
            let tied_targets = tally
                .iter()
                .filter(|(_k, v)| *v == top_vote_count)
                .map(|(k, _v)| k);
            let tied_count = tied_targets.cloned().count();
            let is_voting_stalemate = self.is_voting_stalemate();

            self.previous = Some(Box::new(self.clone()));

            self.phase =
                if is_voting_stalemate || tied_count == 1 && top_target == &VOTE_OPTION_NOBODY {
                    self.transition_message = if is_voting_stalemate {
                        format!("No change in votes 2 rounds in a row. Moving to night time...")
                    } else {
                        format!("Most popular vote was not to lynch. Moving to night time...")
                    };
                    // Move to night
                    GamePhase::Night {
                        count: *count,
                        actions: Vec::new(),
                    }
                } else if tied_count == 1 {
                    let defendant_username = &self.get_player(*top_target).unwrap().username;
                    self.transition_message =
                        format!("Now begins the trial for {defendant_username}:",);
                    // Move to trial
                    GamePhase::Trial {
                        count: *count,
                        defendant: *top_target,
                    }
                } else {
                    self.transition_message =
                        format!("Multiple options were tied for first place. Moving to re-vote");
                    // Move to re-vote
                    GamePhase::Voting {
                        count: *count,
                        poll_id_map: HashMap::new(),
                        vote_options: self.get_vote_targets().collect::<Vec<_>>(),
                        votes: HashMap::new(),
                    }
                };

            Ok(())
        } else {
            Err("Internal error: end_voting caleld when not GamePhase::Voting")
        }
    }
}

#[derive(Clone)]
pub enum Action {
    Kill { source: ChatId, target: ChatId },
}

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
