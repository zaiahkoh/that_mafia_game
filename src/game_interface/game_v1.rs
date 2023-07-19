use crate::game_interface::{Game, GamePhase, Player};
use std::slice;

use super::Action;

#[derive(Clone)]

pub struct GameV1 {
    pub players: Vec<Player>,
    pub phase: GamePhase,
    previous: Option<Box<GameV1>>,
    transition_message: String,
}

impl Game for GameV1 {
    fn from_lobby(lobby: &crate::lobby_manager::Lobby) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn get_players(&mut self) -> slice::IterMut<Player> {
        todo!()
    }

    fn get_phase(&self) -> GamePhase {
        todo!()
    }

    fn end_phase(&mut self) -> Option<GamePhase> {
        todo!()
    }

    fn get_transition_message(&self) -> String {
        todo!()
    }

    fn add_night_action(&mut self, action: Action) -> Result<(), &'static str> {
        todo!()
    }

    fn get_vote_options(&self) -> slice::IterMut<teloxide::types::ChatId> {
        todo!()
    }

    fn get_voters(&self) -> slice::IterMut<Player> {
        todo!()
    }

    fn add_vote_msg_ids(
        &mut self,
        poll_msg_ids: std::collections::HashMap<
            teloxide::types::ChatId,
            teloxide::types::MessageId,
        >,
    ) {
        todo!()
    }

    fn add_votes(&mut self, voter_id: teloxide::types::ChatId, chosen: Vec<i32>) {
        todo!()
    }

    fn get_verdict_options(&self) -> slice::Iter<String> {
        todo!()
    }

    fn get_jury(&self) -> slice::IterMut<Player> {
        todo!()
    }

    fn add_verdict(&mut self) {
        todo!()
    }
}
