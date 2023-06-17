use std::{collections::btree_set::Iter, future::Future, rc::Rc, sync::Arc};

use log::debug;
use teloxide::{
    payloads::SendMessage,
    prelude::*,
    requests::JsonRequest,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    RequestError, dispatching::UpdateFilterExt, dptree::di::Injectable,
};
use tokio::task::JoinSet;

use crate::game_manager::{Game, GameManager, GamePhase, Player};

use super::AsyncBotState;

/*
1. Check player is in a game
2. Check the state of the game (time of day)
3. Check the player's role
 */

async fn gg_rip(
    bot: Bot,
    q: CallbackQuery
) -> Result<(), RequestError> {
    bot.send_message(q.from.id, "GG RIP").await?;
    Ok(())
}

pub fn get_game_handler() -> Handler<
    'static,
    DependencyMap,
    Result<(), teloxide::RequestError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    dptree::entry().branch(
        Update::filter_callback_query().branch(
            dptree::filter(|q: CallbackQuery, bot_state: AsyncBotState| {
                matches!(
                    bot_state
                        .lock()
                        .unwrap()
                        .game_manager
                        .get_player_game(q.from.id.into()),
                    // Some(Game {
                    //     phase: GamePhase::Night { .. },
                    //     ..
                    // })
                    Some(_)
                )
            }
        ).endpoint(handle_night)
    )).branch(Update::filter_callback_query().endpoint(gg_rip))

    // dptree::entry().branch(Update::filter_message())
    // Update::filter_message()
    //     .filter(|msg: Message, bot_state: AsyncBotState| {
    //         bot_state
    //             .lock()
    //             .unwrap()
    //             .game_manager
    //             .get_player_game(msg.chat.id)
    //             .is_some()
    //     })
    //     .branch(dptree::filter(|msg: Message, bot_state: AsyncBotState| {
    //         matches!(
    //             bot_state
    //                 .lock()
    //                 .unwrap()
    //                 .game_manager
    //                 .get_player_game(msg.chat.id),
    //             Some(Game {
    //                 phase: GamePhase::Night { .. },
    //                 ..
    //             })
    //         )
    //     }))
}

// async fn game_handler(
//     bot_state: AsyncBotState,
//     bot: Bot,
//     msg: Message,
// ) -> Result<(), teloxide::RequestError> {
//     let state_lock = bot_state.lock().unwrap();

//     let game = state_lock
//         .game_manager
//         .get_player_game(msg.chat.id)
//         .unwrap();
//     match &game.phase {
//         GamePhase::Night { .. } => handle_night(game, bot, msg),
//         GamePhase::Trial { .. } => handle_voting(),
//         GamePhase::Voting { .. } => handle_trial(),
//     };
//     !todo!();
// }

fn make_player_keyboard(game: &Game) -> InlineKeyboardMarkup {
    let mut keyboard = vec![];

    for player in game.players.iter() {
        let row = vec![InlineKeyboardButton::callback(
            player.username.to_string(),
            player.player_id.to_string(),
        )];
        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

pub async fn start_night(game: &Game, bot: Bot) -> Result<(), &'static str> {
    let mut set = JoinSet::new();

    for player in game.players.iter() {
        let temp = bot.clone();
        let id = player.player_id;
        let shared_game = Arc::new(game.clone());
        set.spawn(async move {
            temp.send_message(id, "Hello everynyan again")
                .reply_markup(make_player_keyboard(&shared_game))
                .await
        });
    }

    while let Some(join_res) = set.join_next().await {
        match join_res {
            Ok(tele_res) => {
                if let Err(_) = tele_res {
                    return Err("Failed to send starting message");
                }
            }
            Err(_) => {
                return Err("Internal Error: join error");
            }
        }
    }

    Ok(())
}

async fn handle_night(
    bot: Bot,
    q: CallbackQuery
) -> Result<(), RequestError> {
    if let Some(target) = q.data {
        let text = format!("You chose: {target}");

        bot.answer_callback_query(q.id).await?;

        if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }
    }

    Ok(())
}

// fn handle_night(game: &Game, bot: Bot, msg: Message) -> Result<(), teloxide::RequestError> {
//     if let Some(player) = game.get_player(msg.chat.id) {
//         if !player.is_alive {
//             return Ok(());
//         }
//     }

//     Ok(())
// }

fn handle_voting() -> Result<(), teloxide::RequestError> {
    todo!()
}

fn handle_trial() -> Result<(), teloxide::RequestError> {
    todo!()
}
