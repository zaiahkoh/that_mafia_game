use std::error::Error;
use teloxide::prelude::*;

use crate::handlers::lobby_handler::get_lobby_handler;
use crate::handlers::main_menu_handler::get_main_menu_handler;
use crate::handlers::new_async_bot_state;

mod handlers;
mod lobby_manager;
mod game_manager;

pub async fn start_mafia_bot() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting That Mafia Game Bot");

    let bot_state = new_async_bot_state();

    let handler = Update::filter_message()
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
