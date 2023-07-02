use super::AsyncBotState;
use crate::game_manager::{Action, Game, GameManager, GamePhase, Role};
use std::{collections::HashMap, sync::Arc};
use teloxide::{
    dispatching::UpdateFilterExt,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, Poll},
    RequestError,
};
use tokio::task::JoinSet;

/*
1. Check player is in a game
2. Check the state of the game (time of day)
3. Check the player's role
 */

async fn gg_rip(bot: Bot, q: CallbackQuery) -> Result<(), RequestError> {
    bot.send_message(q.from.id, "GG RIP").await?;
    Ok(())
}

pub fn get_game_handler() -> Handler<
    'static,
    DependencyMap,
    Result<(), teloxide::RequestError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    dptree::entry()
        .branch(
            Update::filter_callback_query().branch(
                dptree::filter(|q: CallbackQuery, bot_state: AsyncBotState| {
                    matches!(
                        bot_state
                            .lock()
                            .unwrap()
                            .game_manager
                            .get_player_game(q.from.id.into()),
                        Some(Game {
                            phase: GamePhase::Night { .. },
                            ..
                        })
                    )
                })
                .endpoint(handle_night),
            ),
        )
        .branch(Update::filter_callback_query().endpoint(gg_rip))
        .branch(
            Update::filter_poll_answer()
                .filter(|bot_state: AsyncBotState, poll_answer: PollAnswer| {
                    matches!(
                        bot_state
                            .lock()
                            .unwrap()
                            .game_manager
                            .get_player_game(poll_answer.user.id.into()),
                        Some(Game {
                            phase: GamePhase::Voting { .. },
                            ..
                        })
                    )
                })
                .endpoint(handle_vote),
        )
}

async fn test_poll_handler(
    bot_state: AsyncBotState,
    bot: Bot,
    poll_answer: PollAnswer,
) -> Result<(), RequestError> {
    bot.send_message(poll_answer.user.id, "whatsapp").await?;

    Ok(())
}

fn make_player_keyboard(game: &Game) -> InlineKeyboardMarkup {
    let mut keyboard = vec![];

    for player in game.players.iter() {
        let row = vec![InlineKeyboardButton::callback(
            player.username.to_string(),
            player.chat_id.to_string(),
        )];
        keyboard.push(row);
    }

    let none_option = vec![InlineKeyboardButton::callback("No target", "-1")];
    keyboard.push(none_option);

    InlineKeyboardMarkup::new(keyboard)
}

pub async fn start_night(game: &Game, bot: Bot) -> Result<(), &'static str> {
    let mut message_set = JoinSet::new();

    // Send starting messages
    for player in game.players.iter() {
        let temp = bot.clone();
        let chat_id = player.chat_id;
        let role = player.role;
        message_set.spawn(async move {
            temp.send_message(
                chat_id,
                format!("Good evening everynyan. You are a {:?}", role),
            )
            .await
        });
    }

    while let Some(join_res) = message_set.join_next().await {
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

    // Send targetting messages
    for player in game
        .players
        .iter()
        .filter(|p| matches!(p.role, Role::Mafia))
    {
        let temp = bot.clone();
        let chat_id = player.chat_id;
        let keyboard = make_player_keyboard(game);
        message_set.spawn(async move {
            temp.send_message(chat_id, "Pick a target: ")
                .reply_markup(keyboard)
                .await
        });
    }

    while let Some(join_res) = message_set.join_next().await {
        match join_res {
            Ok(tele_res) => {
                if let Err(_) = tele_res {
                    return Err("Failed to send targetting message");
                }
            }
            Err(_) => {
                return Err("Internal Error: join error");
            }
        }
    }

    Ok(())
}

const NO_TARGET: ChatId = ChatId(-1);

async fn handle_night(
    bot_state: AsyncBotState,
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), RequestError> {
    // Add night_action to game
    let source_id = ChatId::from(q.from.id);
    let target_id = ChatId(q.data.as_ref().unwrap().parse::<i64>().unwrap());

    let mut game_snapshot = None;
    {
        // Wrap code in braces to release lock on bot_state
        let mut state_lock = bot_state.lock().unwrap();

        let game = state_lock
            .game_manager
            .get_player_game(q.from.id.into())
            .unwrap();

        game.push_night_action(Action::Kill {
            source: source_id,
            target: target_id,
        });

        game_snapshot = Some(game.clone());
    }

    // Answer callback query
    let game = game_snapshot.as_ref().unwrap();
    bot.answer_callback_query(q.id).await?;
    if let Some(Message { id, chat, .. }) = q.message {
        let target_username = if target_id == NO_TARGET {
            String::from("No target")
        } else {
            game.get_player(target_id).unwrap().username.clone()
        };
        bot.edit_message_text(chat.id, id, format!("You chose: {target_username}"))
            .await?;
    }

    // Check whether to end night
    let pending_player_count = game.count_night_pending_players().unwrap();
    if pending_player_count == 0 {
        {
            let mut state_lock = bot_state.lock().unwrap();
            let game = state_lock
                .game_manager
                .get_player_game(q.from.id.into())
                .unwrap();
            game.end_night();
            game_snapshot = Some(game.clone());
        }
        let chat_id = game_snapshot.unwrap().players.first().unwrap().chat_id;

        start_voting(chat_id, bot, bot_state).await;
    }

    Ok(())
}

async fn start_voting(
    host_id: ChatId,
    bot: Bot,
    bot_state: AsyncBotState,
) -> Result<(), &'static str> {
    let mut game_snapshot = None;
    {
        let mut state_lock = bot_state.lock().unwrap();
        let game = state_lock.game_manager.get_player_game(host_id).unwrap();
        game_snapshot = Some(game.clone());
    }

    let game = game_snapshot.unwrap();
    let mut message_set = JoinSet::new();

    let mut votable_usernames = game
        .get_vote_options()
        .unwrap()
        .iter()
        .map(|x| x.1.clone())
        .collect::<Vec<_>>();

    for player in game.players.iter() {
        let temp = bot.clone();
        let chat_id = player.chat_id;
        let transition_message = game.get_transition_message();
        let option_text = votable_usernames.clone();

        message_set.spawn(async move {
            temp.send_message(chat_id, transition_message).await;

            let poll_res = temp
                .send_poll(
                    chat_id,
                    format!("What is your favourite color"),
                    option_text,
                )
                .allows_multiple_answers(true)
                .is_anonymous(false)
                .await;
            (chat_id, poll_res)
        });
    }

    let mut poll_id_map = HashMap::new();
    while let Some(join_res) = message_set.join_next().await {
        match join_res {
            Ok((chat_id, tele_res)) => match tele_res {
                Ok(message) => {
                    poll_id_map.insert(chat_id, message.id);
                }
                Err(err) => {
                    panic!("{err}");
                }
            },
            Err(err) => {
                panic!("{err}");
            }
        };
    }

    bot_state
        .lock()
        .unwrap()
        .game_manager
        .get_player_game(host_id)
        .unwrap()
        .add_poll_id_map(poll_id_map);

    Ok(())
}

/*
1. If skip, skip
2. If PollAnswer, add to results and close poll
3. Check for finalised answer.
    a. If tied and same as previous, skip to night
    b. If tied and different, re-vote
    c. If outcome, then go to trial
 */

async fn handle_vote(
    bot_state: AsyncBotState,
    bot: Bot,
    poll_answer: PollAnswer,
) -> Result<(), teloxide::RequestError> {
    let chat_id = ChatId::from(poll_answer.user.id);
    let mut message_id_opt = None;
    let mut target_username_opt = None;

    // Add votes to game
    {
        let state_lock = &mut bot_state.lock().unwrap();
        let game = state_lock.game_manager.get_player_game(chat_id).unwrap();
        target_username_opt = Some(game.add_votes(chat_id, poll_answer.option_ids).unwrap());
        message_id_opt = Some(game.get_voter_poll_msg_id(chat_id).unwrap());
    }

    if let Some(message_id) = message_id_opt {
        bot.stop_poll(poll_answer.user.id, message_id).await?;
    }

    bot.send_message(
        poll_answer.user.id,
        format!("You voted for: {:?}", target_username_opt.unwrap()),
    )
    .await?;
    Ok(())
}

async fn start_trial(
    host_id: ChatId,
    bot: Bot,
    bot_state: AsyncBotState,
) -> Result<(), &'static str> {
    todo!()
}

fn handle_trial() -> Result<(), teloxide::RequestError> {
    todo!()
}
