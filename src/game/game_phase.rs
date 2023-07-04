use std::{collections::HashMap, fmt};
use teloxide::types::{ChatId, MessageId};

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
pub const VOTE_OPTION_ABSTAIN: ChatId = ChatId(-2);

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
