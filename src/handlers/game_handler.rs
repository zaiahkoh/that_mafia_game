use std::collections::HashMap;
use teloxide::{
    dispatching::UpdateFilterExt,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
    RequestError,
};
use tokio::task::JoinSet;

use super::AsyncBotState;
use crate::{game::*, game_manager::GameManager};

pub fn get_game_handler() -> Handler<
    'static,
    DependencyMap,
    Result<(), teloxide::RequestError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    let is_night_action = |q: CallbackQuery, bot_state: AsyncBotState| {
        let mut state_lock = bot_state.lock().unwrap();
        let opt = state_lock.game_manager.get_player_game(q.from.id.into());

        if let Some(game) = opt {
            matches!(game.get_phase(), GamePhase::Night { .. })
        } else {
            false
        }
    };
    let is_voting_vote = |bot_state: AsyncBotState, poll_answer: PollAnswer| {
        let mut state_lock = bot_state.lock().unwrap();
        let opt = state_lock
            .game_manager
            .get_player_game(poll_answer.user.id.into());

        if let Some(game) = opt {
            matches!(game.get_phase(), GamePhase::Voting { .. })
        } else {
            false
        }
    };
    let is_trial_verdict = |bot_state: AsyncBotState, poll_answer: PollAnswer| {
        let mut state_lock = bot_state.lock().unwrap();
        let opt = state_lock
            .game_manager
            .get_player_game(poll_answer.user.id.into());

        if let Some(game) = opt {
            matches!(game.get_phase(), GamePhase::Trial { .. })
        } else {
            false
        }
    };

    let is_in_game = |msg: Message, bot_state: AsyncBotState| {
        bot_state
            .lock()
            .unwrap()
            .game_manager
            .get_player_game(msg.chat.id)
            .is_some()
    };

    dptree::entry()
        .branch(
            Update::filter_callback_query()
                .branch(dptree::filter(is_night_action).endpoint(handle_night)),
        )
        .branch(
            Update::filter_poll_answer()
                .filter(is_voting_vote)
                .endpoint(handle_vote),
        )
        .branch(
            Update::filter_poll_answer()
                .filter(is_trial_verdict)
                .endpoint(handle_trial),
        )
        .branch(
            Update::filter_message()
                .filter(is_in_game)
                .endpoint(no_response_handler),
        )
}

async fn no_response_handler() -> Result<(), RequestError> {
    Ok(())
}

/// `options` should be a vector of (text: String, data: String)
fn make_keyboard(options: Vec<(ChatId, String)>) -> InlineKeyboardMarkup {
    let keyboard = options
        .iter()
        .map(|(chat_id, username)| {
            vec![InlineKeyboardButton::callback(
                username,
                chat_id.to_string(),
            )]
        })
        .collect::<Vec<_>>();

    InlineKeyboardMarkup::new(keyboard)
}

async fn start_next_phase(
    phase_opt: Option<GamePhase>,
    chat_id: ChatId,
    bot: Bot,
    bot_state: AsyncBotState,
) -> Result<(), &'static str> {
    match phase_opt {
        Some(GamePhase::Night { .. }) => start_night(chat_id, bot, bot_state).await,
        Some(GamePhase::Trial { .. }) => start_trial(chat_id, bot, bot_state).await,
        Some(GamePhase::Voting { .. }) => start_voting(chat_id, bot, bot_state).await,
        Some(GamePhase::Ending) => start_ending(chat_id, bot, bot_state).await,
        None => Ok(()),
    }
}

pub async fn start_night(
    chat_id: ChatId,
    bot: Bot,
    bot_state: AsyncBotState,
) -> Result<(), &'static str> {
    let game = {
        let mut state_lock = bot_state.lock().unwrap();
        let game = state_lock.game_manager.get_player_game(chat_id).unwrap();
        game.snapshot()
    };
    let mut message_set = JoinSet::new();

    // Queue transition messages
    for player in game.get_players() {
        let bot_clone = bot.clone();
        let chat_id = player.chat_id;
        let text = game.get_transition_message();
        message_set.spawn(async move { bot_clone.send_message(chat_id, text).await });
    }

    // Queue targetting messages
    let night_actions = game.get_night_actions();
    for (chat_id, (message, options)) in night_actions {
        let bot_clone = bot.clone();
        if options.len() > 0 {
            let keyboard = make_keyboard(options.to_vec());
            message_set.spawn(async move {
                bot_clone
                    .send_message(chat_id, message)
                    .reply_markup(keyboard)
                    .await
            });
        } else {
            message_set.spawn(async move { bot_clone.send_message(chat_id, message).await });
        }
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

async fn handle_night(
    bot_state: AsyncBotState,
    bot: Bot,
    q: CallbackQuery,
) -> Result<(), RequestError> {
    // Add night_action to game
    let source_id = ChatId::from(q.from.id);
    let target_id = ChatId(q.data.as_ref().unwrap().parse::<i64>().unwrap());

    let (game, phase_opt) = {
        // Wrap code in braces to release lock on bot_state
        let mut state_lock = bot_state.lock().unwrap();

        let game = state_lock
            .game_manager
            .get_player_game(q.from.id.into())
            .unwrap();
        game.add_night_action(source_id, target_id);

        (game.snapshot(), game.end_phase().cloned())
    };

    // Answer callback query
    bot.answer_callback_query(q.id).await?;
    if let Some(Message { id, chat, .. }) = q.message {
        let chosen_text = game
            .get_night_actions()
            .get(&source_id)
            .unwrap()
            .1
            .iter()
            .find(|(chat_id, _username)| *chat_id == target_id)
            .unwrap()
            .1
            .clone();

        bot.edit_message_text(chat.id, id, format!("You chose: {chosen_text}"))
            .await?;
    }

    start_next_phase(phase_opt, source_id, bot, bot_state).await;

    Ok(())
}

async fn start_voting(
    host_id: ChatId,
    bot: Bot,
    bot_state: AsyncBotState,
) -> Result<(), &'static str> {
    let game = {
        let mut state_lock = bot_state.lock().unwrap();
        let game = state_lock.game_manager.get_player_game(host_id).unwrap();
        game.snapshot()
    };

    let mut message_set = JoinSet::new();

    let votable_usernames = game
        .get_vote_options()
        .iter()
        .map(|x| x.1.clone())
        .collect::<Vec<_>>();

    for player in game.get_voters() {
        let temp = bot.clone();
        let chat_id = player.chat_id;
        let transition_message = game.get_transition_message().clone();
        let option_text = votable_usernames.clone();

        message_set.spawn(async move {
            temp.send_message(chat_id, transition_message).await;

            let poll_res = temp
                .send_poll(chat_id, format!("Who to put on trial?"), option_text)
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
        .add_poll_msg_ids(poll_id_map);

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

    // Add votes to game
    let (message_id, phase_opt) = {
        let state_lock = &mut bot_state.lock().unwrap();
        let game = state_lock.game_manager.get_player_game(chat_id).unwrap();
        (
            *game.get_poll_msg_ids().get(&chat_id).unwrap(),
            game.end_phase().cloned(),
        )
    };

    bot.stop_poll(poll_answer.user.id, message_id).await?;

    start_next_phase(phase_opt, chat_id, bot, bot_state).await;

    Ok(())
}

async fn start_trial(
    host_id: ChatId,
    bot: Bot,
    bot_state: AsyncBotState,
) -> Result<(), &'static str> {
    let game = {
        let mut state_lock = bot_state.lock().unwrap();
        let game = state_lock.game_manager.get_player_game(host_id).unwrap();
        game.snapshot()
    };

    let verdict_option_texts = game
        .get_verdict_options()
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>();

    let mut message_set = JoinSet::new();
    for player in game.get_voters() {
        let bot_clone = bot.clone();
        let chat_id = player.chat_id;
        let transition_message = game.get_transition_message().clone();
        let options_text = verdict_option_texts.clone();
        message_set.spawn(async move {
            bot_clone.send_message(chat_id, transition_message).await;

            let poll_res = bot_clone
                .send_poll(chat_id, format!("Vote on trial: "), options_text)
                .is_anonymous(true)
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
        .add_poll_msg_ids(poll_id_map);

    Ok(())
}

async fn handle_trial(
    bot_state: AsyncBotState,
    bot: Bot,
    poll_answer: PollAnswer,
) -> Result<(), teloxide::RequestError> {
    let chat_id = ChatId::from(poll_answer.user.id);
    assert_eq!(
        poll_answer.option_ids.len(),
        1,
        "Internal error: trial poll options_ids.len != 1"
    );
    let chosen_id = poll_answer.option_ids.first().unwrap();

    // Add verdict to game
    let (message_id, phase_opt) = {
        let state_lock = &mut bot_state.lock().unwrap();
        let game = state_lock.game_manager.get_player_game(chat_id).unwrap();
        game.add_verdict(chat_id, *chosen_id);
        (
            *game.get_poll_msg_ids().get(&chat_id).unwrap(),
            game.end_phase().cloned(),
        )
    };

    // Stop poll
    bot.stop_poll(poll_answer.user.id, message_id).await?;

    start_next_phase(phase_opt, chat_id, bot, bot_state).await;

    Ok(())
}

async fn start_ending(
    host_id: ChatId,
    bot: Bot,
    bot_state: AsyncBotState,
) -> Result<(), &'static str> {
    let game_snapshot = {
        let mut state_lock = bot_state.lock().unwrap();
        state_lock.game_manager.remove_game(host_id)
    };
    let game = game_snapshot.unwrap();

    let mut message_set = JoinSet::new();

    let ending_message = game.get_transition_message();
    for player in game.get_players() {
        let bot_clone = bot.clone();
        let chat_id = player.chat_id;
        let text = ending_message.clone();

        message_set.spawn(async move { bot_clone.send_message(chat_id, text).await });
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
