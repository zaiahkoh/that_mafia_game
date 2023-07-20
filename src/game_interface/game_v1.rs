use teloxide::types::{ChatId, MessageId};

use crate::game_interface::{Game, GamePhase, Player};
use std::{
    collections::{HashMap, HashSet},
    slice,
};

use super::{Action, Role, Verdict, NOBODY_CHAT_ID, NOBODY_USERNAME, VOTE_OPTION_NOBODY};

#[derive(Clone)]

pub struct GameV1 {
    pub players: Vec<Player>,
    pub phase: GamePhase,
    previous: Option<Box<GameV1>>,
    transition_message: String,
}

impl GameV1 {
    fn should_end_night(&self) -> bool {
        if let GamePhase::Night { actions, .. } = &self.phase {
            let check_player_is_idle = |p: &&Player| match p.role {
                Role::Mafia => actions
                    .iter()
                    .find(|a| match a {
                        Action::Kill { source, .. } if source == &p.chat_id => true,
                        _ => false,
                    })
                    .is_none(),
                Role::Civilian => false,
            };

            let idle_players = self.players.iter().filter(check_player_is_idle);

            idle_players.count() == 0
        } else {
            panic!("should_end_night called when not in GamePhase::Night")
        }
    }

    /// Attempts to end the night, returning the new, current phase if successful
    fn end_night(&self) -> Option<&GamePhase> {
        if let GamePhase::Night { actions, .. } = &self.phase {
            if !self.should_end_night() {
                return None;
            }

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
                votes: HashMap::new(),
                poll_id_map: HashMap::new(),
            };

            Some(&self.phase)
        } else {
            panic!("end_night called when not in GamePhase::Night")
        }
    }

    fn should_end_voting(&self) -> bool {
        if let GamePhase::Voting { votes, .. } = &self.phase {
            let voters = self.players.iter().filter(|p| p.is_alive);

            let idle_voters = voters.filter(|p| !votes.contains_key(&p.chat_id));

            idle_voters.count() == 0
        } else {
            panic!("should_end_voting called when not in GamePhase::Voting")
        }
    }

    fn end_voting(&self) -> Option<&GamePhase> {
        if let GamePhase::Voting { votes, .. } = &self.phase {
            if !self.should_end_voting() {
                return None;
            }

            let mut tally = HashMap::new();
            for v in self.get_vote_options() {
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
            let tied_count = tied_targets.count();
            let is_voting_stalemate = self.is_voting_stalemate();

            self.previous = Some(Box::new(self.clone()));

            (self.phase, self.transition_message) = if is_voting_stalemate {
                (
                    GamePhase::Night {
                        actions: Vec::new(),
                    },
                    format!("No change in votes 2 rounds in a row. Moving to night time..."),
                )
            } else if tied_count == 1 && top_target == &VOTE_OPTION_NOBODY {
                (
                    GamePhase::Night {
                        actions: Vec::new(),
                    },
                    format!("Most popular vote was not to lynch. Moving to night time..."),
                )
            } else if tied_count == 1 {
                let defendant_username = &self.get_player(*top_target).unwrap().username;
                (
                    GamePhase::Trial {
                        defendant_id: *top_target,
                        poll_id_map: HashMap::new(),
                        verdicts: HashMap::new(),
                    },
                    format!("Now begins the trial for {defendant_username}:",),
                )
            } else {
                (
                    GamePhase::Voting {
                        poll_id_map: HashMap::new(),
                        votes: HashMap::new(),
                    },
                    format!("Multiple options were tied for first place. Moving to re-vote"),
                )
            };

            return Some(&self.phase);
        } else {
            panic!("end_voting called when not in GamePhase::Voting")
        }
    }

    fn should_end_trial(&self) -> bool {
        if let GamePhase::Trial {
            verdicts,
            defendant_id: defendant,
            ..
        } = &self.phase
        {
            let jurors = self
                .players
                .iter()
                .filter(|p: &&Player| p.is_alive && p.chat_id != *defendant);

            let idle_jurors = jurors.filter(|p| !verdicts.contains_key(&p.chat_id));

            idle_jurors.count() == 0
        } else {
            panic!("should_end_trial called when not in GamePhase::Trial")
        }
    }

    fn end_trial(&mut self) -> Option<&GamePhase> {
        if let GamePhase::Trial {
            defendant_id: defendant,
            poll_id_map,
            verdicts,
        } = &mut self.phase
        {
            if !self.should_end_trial() {
                return None;
            }

            let guilties = verdicts
                .values()
                .filter(|v| matches!(v, Verdict::Guilty))
                .count();
            let innocents = verdicts
                .values()
                .filter(|v| matches!(v, Verdict::Innocent))
                .count();

            let username = self.get_player(*defendant).unwrap().username.clone();

            self.transition_message = if guilties >= innocents {
                let victim = self
                    .players
                    .iter_mut()
                    .find(|p| p.chat_id == *defendant)
                    .unwrap();
                victim.is_alive = false;

                format!(
                    "By a vote of {guilties} guilty to {innocents} innocent, {} was lynched",
                    username
                )
            } else {
                format!(
                    "By a vote of {innocents} innocent to {guilties} guilty, {} was released",
                    username
                )
            };

            Some(&self.phase)
        } else {
            panic!("end_trial called when not in GamePhase::Trial")
        }
    }

    fn is_voting_stalemate(&self) -> bool {
        if let GamePhase::Voting { votes, .. } = &self.phase {
            if let Some(inner) = &self.previous {
                if let GameV1 {
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

    fn get_player(&self, chat_id: ChatId) -> Option<&Player> {
        self.players.iter().find(|p| p.chat_id == chat_id)
    }
}

impl Game for GameV1 {
    fn from_lobby(lobby: &crate::lobby_manager::Lobby) -> Self
    where
        Self: Sized,
    {
        GameV1 {
            players: vec![],
            phase: GamePhase::Night {
                actions: Vec::new(),
            },
            previous: None,
            transition_message: String::from("Welcome to the Mafia Game"),
        }
    }

    fn snapshot(&self) -> Box<dyn Game> {
        Box::new(self.clone())
    }

    fn get_players(&self) -> slice::Iter<Player> {
        self.players.iter()
    }

    fn get_phase(&self) -> &GamePhase {
        &self.phase
    }

    fn end_phase(&mut self) -> Option<&GamePhase> {
        match &mut self.phase {
            GamePhase::Night { .. } => self.end_night(),
            GamePhase::Voting { .. } => self.end_voting(),
            GamePhase::Trial { .. } => self.end_trial(),
        }
    }

    fn get_transition_message(&self) -> String {
        self.transition_message.clone()
    }

    fn get_night_actions(&self) -> HashMap<ChatId, (String, Vec<(ChatId, String)>)> {
        let mut result = HashMap::new();
        for p in self.players.iter() {
            if !p.is_alive {
                continue;
            }
            let (text, targets) = match p.role {
                Role::Mafia => {
                    let base_options = vec![(NOBODY_CHAT_ID, NOBODY_USERNAME)].iter().map(|p| *p);
                    let options = self
                        .players
                        .iter()
                        .filter(|p| p.is_alive && p.role != Role::Mafia)
                        .map(|p| (p.chat_id, p.username.clone()))
                        .chain(base_options)
                        .collect();
                    (
                        String::from("You are a Mafia. Pick a victim to kil:"),
                        options,
                    )
                }
                Role::Civilian => (String::from("You are a Civilian"), Vec::new()),
            };
            result.insert(p.chat_id, (text, targets));
        }

        result
    }

    fn add_night_action(&mut self, actor_id: ChatId, target_id: ChatId) {
        if let GamePhase::Night { actions, .. } = &mut self.phase {
            let is_valid_target = self
                .get_night_actions()
                .get(&actor_id)
                .unwrap()
                .1
                .iter()
                .find(|(chat_id, _)| *chat_id == target_id)
                .is_some();
            if !is_valid_target {
                panic!("add_night_action called with illegal target_id");
            }

            let actor_role = self.get_player(actor_id).unwrap().role;
            match actor_role {
                Role::Mafia => actions.push(Action::Kill {
                    source: actor_id,
                    target: target_id,
                }),
                _ => {}
            }
        } else {
            panic!("add_night_action called when not GamePhase::Night")
        }
    }

    fn get_vote_options(&self) -> slice::Iter<(ChatId, String)> {
        if let GamePhase::Voting { .. } = self.phase {
            let base_options = vec![(VOTE_OPTION_NOBODY, String::from("Nobody"))];

            self.players
                .iter()
                .filter(|p| p.is_alive)
                .map(|p| (p.chat_id, p.username.clone()))
                .chain(base_options)
                .collect::<Vec<_>>()
                .iter()
        } else {
            panic!("get_vote_options called when not in GamePhase::Voting")
        }
    }

    fn get_voters(&self) -> slice::Iter<Player> {
        if let GamePhase::Voting { .. } = self.phase {
            self.players
                .iter()
                .filter(|p| p.is_alive)
                .map(|p| *p)
                .collect::<Vec<_>>()
                .iter()
        } else {
            panic!("get_voters called when not in GamePhase::Voting")
        }
    }

    fn add_vote_msg_ids(&mut self, poll_msg_ids: HashMap<ChatId, MessageId>) {
        if let GamePhase::Voting { poll_id_map, .. } = &mut self.phase {
            poll_msg_ids.clone_into(poll_id_map);
        } else {
            panic!("add_vote_msg_ids called when not in GamePhase::Voting")
        }
    }

    fn add_vote(&mut self, voter_id: teloxide::types::ChatId, choices: Vec<i32>) {
        if let GamePhase::Voting { votes, .. } = &mut self.phase {
            let vote_options = self.get_vote_options().collect::<Vec<_>>();
            let chosen_ids = choices
                .iter()
                .map(|i| vote_options[*i as usize].0)
                .collect::<Vec<_>>();

            votes.insert(voter_id, chosen_ids);
        } else {
            panic!("add_vote called when not in GamePhase::Voting")
        }
    }

    fn get_verdict_options(&self) -> slice::Iter<Verdict> {
        if let GamePhase::Trial { .. } = self.phase {
            vec![Verdict::Guilty, Verdict::Innocent, Verdict::Abstain].iter()
        } else {
            panic!("get_verdict_options called when not in GamePhase::Trial")
        }
    }

    fn get_jury(&self) -> slice::Iter<Player> {
        if let GamePhase::Trial {
            defendant_id: defendant,
            ..
        } = self.phase
        {
            self.players
                .iter()
                .filter(|p| p.is_alive && p.chat_id != defendant)
                .map(|p| *p)
                .collect::<Vec<_>>()
                .iter()
        } else {
            panic!("get_jury called when not in GamePhase::Trial")
        }
    }

    fn add_verdict(&mut self, juror_id: ChatId, chosen: i32) {
        if let GamePhase::Trial { verdicts, .. } = &mut self.phase {
            verdicts.insert(
                juror_id,
                *self.get_verdict_options().nth(chosen as usize).unwrap(),
            );
        } else {
            panic!("add_verdict called when not in GamePhase::Trial")
        }
    }
}
