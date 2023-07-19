use crate::lobby_manager::Lobby;
use std::{collections::HashMap, fmt, slice};
use teloxide::types::{ChatId, MessageId};

#[derive(Clone)]
pub struct Player {
    pub chat_id: ChatId,
    pub username: String,
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
        poll_id_map: HashMap<ChatId, MessageId>,
        verdicts: HashMap<ChatId, Verdict>,
    },
}

#[derive(Clone)]
pub enum Action {
    Kill { source: ChatId, target: ChatId },
}

pub const VOTE_OPTION_NOBODY: ChatId = ChatId(-1);

#[derive(Clone, Copy)]
pub enum Verdict {
    Guilty,
    Innocent,
    Abstain,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Verdict::Guilty => write!(f, "Guilty"),
            Verdict::Innocent => write!(f, "Innocent"),
            Verdict::Abstain => write!(f, "Abstain"),
        }
    }
}

pub trait Game: Send + Sync {
    /// Creates a Game instace from a lobby
    fn from_lobby(lobby: &Lobby) -> Self
    where
        Self: Sized;

    fn get_players(&self) -> slice::Iter<Player>;

    fn get_phase(&self) -> GamePhase;

    /// Attempts to end the phase. Returns Some(GamePhase) if the phase ended
    fn end_phase(&mut self) -> Option<GamePhase>;

    /// Returns the most recent transition message
    fn get_transition_message(&self) -> String;

    /// Panics if the game is not in GamePhase::Night or if the action is illegal
    fn add_night_action(&mut self, action: Action) -> Result<(), &'static str>;

    /// Panics if the game is not in GamePhase::Voting
    fn get_vote_options(&self) -> slice::IterMut<ChatId>;

    /// Panics if the game is not in GamePhase::Voting
    fn get_voters(&self) -> slice::IterMut<Player>;

    /// Panics if the game is not in GamePhase::Voting. The poll_msg_ids HashMap should map the
    /// voter's chat id to the message id of the poll.
    fn add_vote_msg_ids(&mut self, poll_msg_ids: HashMap<ChatId, MessageId>);

    /// Panics if the game is not in GamePhase::Voting. The chosen vector should contain the index of the options
    /// as they appear in `get_vote_options`.
    fn add_votes(&mut self, voter_id: ChatId, chosen: Vec<i32>);

    /// Panics if the game is not in GamePhase::Trial
    fn get_verdict_options(&self) -> slice::Iter<String>;

    /// Panics if the game is not in GamePhase::Trial
    fn get_jury(&self) -> slice::IterMut<Player>;

    /// Panics if the game is not in GamePhase::Trial
    fn add_verdict(&mut self);
}

pub mod game_v1;
