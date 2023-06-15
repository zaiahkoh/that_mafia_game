use teloxide::prelude::*;

use crate::game_manager::GameManager;

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
}
