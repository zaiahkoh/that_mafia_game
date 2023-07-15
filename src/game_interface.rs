use std::collections::HashMap;

use teloxide::types::{ChatId, MessageId};

use crate::{
    game::game_phase::{Action, GamePhase},
    lobby_manager::Lobby,
};

pub struct Player {
    chat_id: ChatId,
    username: String,
}

pub trait Game {
    fn from_lobby(lobby: &Lobby) -> Self;

    fn get_phase(&self) -> GamePhase;

    /// Attempts to end the phase. Returns Some(GamePhase) if the phase ended
    fn end_phase(&mut self) -> Option<GamePhase>;

    /// Returns the most recent transition message
    fn get_transition_message(&self) -> String;

    /// Panics if the game is not in GamePhase::Night or if the action is illegal
    fn add_night_action(&mut self, action: Action) -> Result<(), &'static str>;

    /// Panics if the game is not in GamePhase::Voting
    fn get_vote_options(&self) -> dyn Iterator<Item = Player>;

    /// Panics if the game is not in GamePhase::Voting
    fn get_voters(&self) -> dyn Iterator<Item = Player>;

    /// Panics if the game is not in GamePhase::Voting. The poll_msg_ids HashMap should map the
    /// voter's chat id to the message id of the poll.
    fn add_vote_msg_ids(&mut self, poll_msg_ids: HashMap<ChatId, MessageId>);

    /// Panics if the game is not in GamePhase::Voting. The chosen vector should contain the index of the options
    /// as they appear in `get_vote_options`.
    fn add_votes(&mut self, voter_id: ChatId, chosen: Vec<i32>);

    /// Panics if the game is not in GamePhase::Trial
    fn get_verdict_options(&self) -> dyn Iterator<Item = String>;

    /// Panics if the game is not in GamePhase::Trial
    fn get_jury(&self) -> dyn Iterator<Item = Player>;

    
}
