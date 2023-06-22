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
            player.player_id.to_string(),
        )];
        keyboard.push(row);
    }

    let none_option = vec![InlineKeyboardButton::callback("No target", "-1")];
    keyboard.push(none_option);

    InlineKeyboardMarkup::new(keyboard)
}

pub async fn start_night(game: &Game, bot: Bot) -> Result<(), &'static str> {
    let mut set = JoinSet::new();

    for player in game.players.iter() {
        let temp = bot.clone();
        let id = player.player_id;
        let shared_game = Arc::new(game.clone());
        set.spawn(async move {
            let role = shared_game.get_role(id).unwrap();
            temp.send_message(id, format!("Good evening everynyan. You are a {:?}", role))
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

    for player in game
        .players
        .iter()
        .filter(|p| matches!(p.role, Role::Mafia))
    {
        let temp = bot.clone();
        let id = player.player_id;
        let shared_game = Arc::new(game.clone());
        set.spawn(async move {
            let role = shared_game.get_role(id).unwrap();
            temp.send_message(id, "Pick a target: ")
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
    bot_state: AsyncBotState,
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), RequestError> {
    if let Some(target) = q.data.as_ref() {
        let text = format!("You chose: {target}");

        bot.answer_callback_query(q.id).await?;

        if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }
    }

    let chat_id = ChatId::from(q.from.id);
    let mut opt = None;
    {
        // Wrap code in braces to release lock on bot_state
        let mut state_lock = bot_state.lock().unwrap();
        let mut game = state_lock
            .game_manager
            .get_player_game(q.from.id.into())
            .unwrap()
            .clone();

        game.push_night_action(Action::Kill {
            source: chat_id,
            target: ChatId(q.data.as_ref().unwrap().parse::<i64>().unwrap()),
        })
        .unwrap();

        state_lock.game_manager.update_game(game.clone(), chat_id);

        opt = Some(game.clone());
    }

    let game = opt.as_ref().unwrap();
    let pending_player_count = game.count_night_pending_players().unwrap();

    bot.send_message(
        chat_id,
        format!("Remaining players: {pending_player_count}"),
    )
    .await?;

    if pending_player_count == 0 {
        let mut temp = None;
        {
            let mut state_lock = bot_state.lock().unwrap();
            let mut game = state_lock
                .game_manager
                .get_player_game(q.from.id.into())
                .as_mut()
                .unwrap()
                .clone();
            game.end_night();
            state_lock.game_manager.update_game(game.clone(), chat_id);
            temp = Some(game);
        }
        start_voting(temp.as_ref().unwrap(), bot, bot_state).await;
    }
    Ok(())
}

// 1. Send out poll
// 2. Save message_ids
async fn start_voting(game: &Game, bot: Bot, bot_state: AsyncBotState) -> Result<(), &'static str> {
    let mut set = JoinSet::new();
    let mut player_text = game
        .get_alive_players()
        .map(|p| p.player_id.0.to_string())
        .collect::<Vec<_>>();
    player_text.push("Nobody".to_owned());
    player_text.push("Abstain".to_string());

    for player in game.players.iter() {
        let temp = bot.clone();
        let chat_id = player.player_id;
        let transition_message = game.get_transition_message();
        let option_text = player_text.clone();

        set.spawn(async move {
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

    let mut new_game = game.clone();

    if let GamePhase::Voting {
        poll_id_map,
        vote_options: poll_options,
        ..
    } = &mut new_game.phase
    {
        while let Some(join_res) = set.join_next().await {
            match join_res {
                Ok((id, tele_res)) => match tele_res {
                    Ok(message) => {
                        poll_id_map.insert(id, message.id);
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
    } else {
        panic!("game was not in voting phase");
    }

    let chat_id = game.players.first().unwrap().player_id;
    bot_state
        .lock()
        .unwrap()
        .game_manager
        .update_game(new_game, chat_id);

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
    let user_id = poll_answer.user.id;

    let mut game_opt = None;
    let mut selected = Vec::<i32>::new();
    let mut message_id_opt = None;
    {
        let state_lock = &mut bot_state.lock().unwrap();
        let game_manager = &mut state_lock.game_manager;
        game_opt = game_manager.get_player_game(poll_answer.user.id.into());

        if let Some(Game {
            phase:
                GamePhase::Voting {
                    poll_id_map,
                    votes,
                    vote_options,
                    ..
                },
            ..
        }) = game_opt.as_mut()
        {
            for choice in poll_answer.option_ids {
                selected.push(choice);

                let target_id = vote_options[choice as usize].0;
                if votes.get(&target_id).is_none() {
                    votes.insert(target_id, 0);
                }
            }

            let respondent_id = &poll_answer.user.id.into();
            message_id_opt = Some(poll_id_map.get(respondent_id).unwrap().clone());
        }
    }

    let poll_id = poll_answer.poll_id;

    if let Some(message_id) = message_id_opt {
        bot.stop_poll(poll_answer.user.id, message_id).await?;
    }

    bot.send_message(poll_answer.user.id, format!("{:?}", selected))
        .await?;
    Ok(())
}

fn handle_trial() -> Result<(), teloxide::RequestError> {
    todo!()
}
