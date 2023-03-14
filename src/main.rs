use log;
use pretty_env_logger;
use std::{error::Error, sync::Mutex};
use std::sync::Arc;
use teloxide::{prelude::*, types::Me};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting ");

    let players = Arc::new(Mutex::new(Vec::<String>::new()));

    let bot = Bot::from_env();
    let handler = Update::filter_message().endpoint(msg_handler);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![players])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

async fn msg_handler(
    bot: Bot,
    players: Arc<Mutex<Vec<String>>>,
    msg: Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    players.lock().unwrap().push(String::from(msg.chat.username().unwrap()));
    bot.send_message(msg.chat.id, format!("{:?}", players.lock().unwrap())).await?;
    Ok(())
}
