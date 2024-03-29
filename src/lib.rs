use std::error::Error;
use teloxide::prelude::*;

use crate::handlers::{
    game_handler::get_game_handler, lobby_handler::get_lobby_handler,
    main_menu_handler::get_main_menu_handler, new_async_bot_state,
};

mod game;
mod game_manager;
mod handlers;
mod lobby_manager;

pub async fn start_mafia_bot() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting That Mafia Game Bot");

    let bot_state = new_async_bot_state();

    let handler = dptree::entry()
        .branch(get_game_handler())
        .branch(get_lobby_handler())
        .branch(get_main_menu_handler());

    let bot = Bot::from_env();
    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![bot_state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}
