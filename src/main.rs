use log;
use pretty_env_logger;
use std::error::Error;
use teloxide::{prelude::*, types::Me};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting ");

    let bot = Bot::from_env();
    let handler = Update::filter_message().endpoint(msg_handler);

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn msg_handler(bot: Bot, msg: Message, me: Me) -> Result<(), Box<dyn Error + Send + Sync>> {
    bot.send_message(msg.chat.id, "Hello").await?;
    Ok(())
}
