use std::collections::btree_set::Iter;

use teloxide::{prelude::*, requests::JsonRequest, payloads::SendMessage};

use crate::game_manager::{Game, GameManager, GamePhase, Player};

use super::AsyncBotState;

/*
1. Check player is in a game
2. Check the state of the game (time of day)
3. Check the player's role
 */

pub fn get_game_handler() -> Handler<
    'static,
    DependencyMap,
    Result<(), teloxide::RequestError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    dptree::filter(|msg: Message, bot_state: AsyncBotState| {
        bot_state
            .lock()
            .unwrap()
            .game_manager
            .get_player_game(msg.chat.id)
            .is_some()
    })
    .endpoint(game_handler)
}

async fn game_handler(
    bot_state: AsyncBotState,
    bot: Bot,
    msg: Message,
) -> Result<(), teloxide::RequestError> {
    let state_lock = bot_state.lock().unwrap();

    let game = state_lock
        .game_manager
        .get_player_game(msg.chat.id)
        .unwrap();
    match &game.phase {
        GamePhase::Night { .. } => handle_night(game, bot, msg),
        GamePhase::Trial { .. } => handle_voting(),
        GamePhase::Voting { .. } => handle_trial(),
    };
    !todo!();
}

pub async fn start_night(game: &Game, bot: Bot) -> Result<(), teloxide::RequestError> {
    for player in game.players.iter() {
        bot.send_message(player.player_id, "Hello everynyan")
            .await?;
    }

    Ok(())

    // game.players
    //     .iter()
    //     .map(|p| bot.send_message(p.player_id, "Hello everynyan"))

    // todo!()
}

fn handle_night(game: &Game, bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
    if let Some(player) = game.get_player(msg.chat.id) {
        if !player.is_alive {
            return Ok(());
        }
    }

    Ok(())
}

fn handle_voting() -> Result<(), teloxide::RequestError> {
    todo!()
}

fn handle_trial() -> Result<(), teloxide::RequestError> {
    todo!()
}
