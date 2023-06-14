use teloxide::{prelude::*, utils::command::BotCommands};

use super::AsyncBotState;
use crate::lobby_manager::LobbyManager;

pub fn get_lobby_handler() -> Handler<
    'static,
    DependencyMap,
    Result<(), teloxide::RequestError>,
    teloxide::dispatching::DpHandlerDescription,
> {
    dptree::filter(|msg: Message, bot_state: AsyncBotState| {
        bot_state
            .lock()
            .unwrap()
            .lobby_manager
            .get_chats_lobby(msg.chat.id)
            .is_some()
    })
    .filter_command::<LobbyCommand>()
    .endpoint(lobby_handler)
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Lobby commands")]
enum LobbyCommand {
    #[command(description = "Shows the message.")]
    Help,
    #[command(description = "Quit lobby")]
    Quit,
}

async fn lobby_handler(
    bot_state: AsyncBotState,
    bot: Bot,
    msg: Message,
    cmd: LobbyCommand,
) -> Result<(), teloxide::RequestError> {
    let text = match cmd {
        LobbyCommand::Help => LobbyCommand::descriptions().to_string(),
        LobbyCommand::Quit => {
            let mut state_lock = bot_state.lock().unwrap();
            state_lock
                .lobby_manager
                .get_chats_lobby(msg.chat.id)
                .unwrap();
            match state_lock.lobby_manager.quit_lobby(msg.chat.id) {
                Ok(lobby_id) => format!("Quit lobby: {}", lobby_id),
                Err(message) => format!("Encountered error: {}", message),
            }
        }
    };

    bot.send_message(msg.chat.id, text).await?;

    Ok(())
}
