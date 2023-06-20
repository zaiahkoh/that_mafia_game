use super::AsyncBotState;
use crate::game_manager::{Action, Game, GameManager, GamePhase, Role};
use std::sync::Arc;
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
                .endpoint(test_poll_handler),
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
                .unwrap()
                .clone();
            game.end_night();
            state_lock.game_manager.update_game(game.clone(), chat_id);
            temp = Some(game);
        }
        start_voting(temp.as_ref().unwrap(), bot).await;
    }
    Ok(())
}

async fn start_voting(game: &Game, bot: Bot) -> Result<(), &'static str> {
    let mut set = JoinSet::new();

    let shared_bot = Arc::new(bot.clone());
    for player in game.players.iter() {
        let temp = bot.clone();
        let id = player.player_id;
        let shared_game = Arc::new(game.clone());

        set.spawn(async move {
            temp.send_message(id, shared_game.get_transition_message())
                // .reply_markup(make_player_keyboard(&shared_game))
                .await
            // temp.forward_message(id, id, asdf).await
        });

        let colors: Vec<String> = vec!["Red".to_string(), "Blue".to_string()];
        let asdf = shared_bot
            .send_poll(id, "What is your favourite color", colors.into_iter())
            .allows_multiple_answers(true)
            .is_anonymous(false)
            .await
            .unwrap()
            .id;
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

/*
1. If skip, skip
2. If PollAnswer, add to results and close poll
3. Check for finalised answer.
    a. If tied and same as previous, skip to night
    b. If tied and different, re-vote
    c. If outcome, then go to trial
 */

fn handle_vote(
    bot_state: AsyncBotState,
    bot: Bot,
    poll_answer: PollAnswer,
) -> Result<(), teloxide::RequestError> {

    Ok(())
}

fn handle_trial() -> Result<(), teloxide::RequestError> {
    todo!()
}
