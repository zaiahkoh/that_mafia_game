use crate::lobby_manager::Lobby;
use std::{collections::HashMap, fmt};
use teloxide::types::{ChatId, MessageId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Role {
    Mafia,
    Civilian,
}

#[derive(Clone)]
pub struct Player {
    pub chat_id: ChatId,
    pub username: String,
    pub role: Role,
    is_alive: bool,
}

#[derive(Clone)]
pub enum GamePhase {
    Night {
        actions: Vec<Action>,
    },
    Voting {
        poll_id_map: HashMap<ChatId, MessageId>,
        votes: HashMap<ChatId, Vec<ChatId>>,
    },
    Trial {
        defendant_id: ChatId,
        poll_id_map: HashMap<ChatId, MessageId>,
        verdicts: HashMap<ChatId, Verdict>,
    },
    Ending,
}

pub const NOBODY_CHAT_ID: ChatId = ChatId(-1);
pub const NOBODY_USERNAME: &str = "Nobody";

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

    fn snapshot(&self) -> Box<dyn Game>;

    fn get_players(&self) -> Vec<&Player>;

    fn get_phase(&self) -> &GamePhase;

    /// Attempts to end the phase. Returns Some(GamePhase) if the phase ended. \
    /// `GamePhase::Night` should always return `None`
    fn end_phase(&mut self) -> Option<&GamePhase>;

    /// Returns the most recent transition message
    fn get_transition_message(&self) -> String;

    /// Returns a mapping from `chat_id` to `(message: String, options: Vec<(target_id: ChatId, username: String)>)`
    ///
    /// The info is used to display an options box for the user. \
    /// The options vector will be either length 0 or >= 2. If zero, then no options will be displayed
    fn get_night_actions(&self) -> HashMap<ChatId, (String, Vec<(ChatId, String)>)>;

    /// Panics if the game is not in GamePhase::Night
    fn add_night_action(&mut self, actor_id: ChatId, target_id: ChatId);

    /// Panics if the game is not in GamePhase::Voting.
    /// Returns a iterator over ChatIds and the corresponding display names.
    fn get_vote_options(&self) -> Vec<(ChatId, String)>;

    /// Panics if the game is not in GamePhase::Voting
    fn get_voters(&self) -> Vec<&Player>;

    /// Panics if the game is not in GamePhase::Voting. The chosen vector should contain the index of the options
    /// as they appear in `get_vote_options`.
    fn add_vote(&mut self, voter_id: ChatId, choices: Vec<i32>);

    /// Panics if the game is not in GamePhase::Trial
    fn get_verdict_options(&self) -> Vec<Verdict>;

    /// Panics if the game is not in GamePhase::Trial
    fn get_jury(&self) -> Vec<&Player>;

    /// Panics if the game is not in GamePhase::Trial.\
    /// The chosen index should correspond to the entry in `get_verdict_options`
    fn add_verdict(&mut self, juror_id: ChatId, chosen: i32);

    /// Panics if the game is not in GamePhase::Voting or GamePhase::Trial.
    /// The poll_msg_ids HashMap should map the voter's chat id to the message id of the poll.
    fn add_poll_msg_ids(&mut self, poll_msg_ids: HashMap<ChatId, MessageId>);

    /// Panics if the game is not in GamePhase::Voting or GamePhase::Trial.
    fn get_poll_msg_ids(&self) -> HashMap<ChatId, MessageId>;
}

pub mod game_v1;
