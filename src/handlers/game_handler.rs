use super::AsyncBotState;
use crate::game_manager::{Game, GameManager, GamePhase};
use std::sync::Arc;
use teloxide::{
    dispatching::UpdateFilterExt,
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup},
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

    InlineKeyboardMarkup::new(keyboard)
}

pub async fn start_night(game: &Game, bot: Bot) -> Result<(), &'static str> {
    let mut set = JoinSet::new();

    for player in game.players.iter() {
        let temp = bot.clone();
        let id = player.player_id;
        let shared_game = Arc::new(game.clone());
        set.spawn(async move {
            temp.send_message(id, "Good evening everynyan")
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
    if let Some(target) = q.data {
        let text = format!("You chose: {target}");

        bot.answer_callback_query(q.id).await?;

        if let Some(Message { id, chat, .. }) = q.message {
            // let temp = bot.clone();
            // tasks.spawn(async move {
            //     temp.edit_message_text(chat.id, id, text).await
            // });
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            let temp = bot.clone();
            // tasks.spawn(async move {
            //     temp.edit_message_text_inline(id, text).await
            // });
            bot.edit_message_text_inline(id, text).await?;
        }
    }

    let mut chat_id = None;
    let mut game: Option<Game> = None;
    {
        let state_lock = bot_state.lock().unwrap();
        game = Some(
            state_lock
                .game_manager
                .get_player_game(q.from.id.into())
                .unwrap()
                .clone(),
        );
        chat_id = Some(ChatId::from(q.from.id));
    }

    bot.send_message(
        chat_id.unwrap(),
        format!(
            "Remaining players: {}",
            game.unwrap().count_night_pending_players().unwrap()
        ),
    )
    .await?;

    Ok(())
}

async fn start_voting(game: &Game, bot: Bot) -> Result<(), &'static str> {
    let mut set = JoinSet::new();

    for player in game.players.iter() {
        let temp = bot.clone();
        let id = player.player_id;
        let shared_game = Arc::new(game.clone());
        set.spawn(async move {
            temp.send_message(id, "Good morning everynyan")
                .reply_markup(make_player_keyboard(&shared_game))
                .await
        });
    }
    Ok(())
}

fn handle_voting() -> Result<(), teloxide::RequestError> {
    todo!()
}

fn handle_trial() -> Result<(), teloxide::RequestError> {
    todo!()
}
