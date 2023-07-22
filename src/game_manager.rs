use teloxide::types::ChatId;

use crate::game::Game;

pub trait GameManager: Send + Sync {
    /// Returns a mutable reference to the chat_id's game, if present
    fn get_player_game(&mut self, chat_id: ChatId) -> Option<&mut Box<dyn Game>>;

    // Adds game to the map
    fn add_game(&mut self, game: Box<dyn Game>);

    // Removes game for the chat_id from the map
    fn remove_game(&mut self, chat_id: ChatId) -> Option<Box<dyn Game>>;

    // If the host quits, then a remaining player should be randomly chosen to be the new host
    fn quit_game(&mut self, chat_id: ChatId) -> Result<&mut dyn Game, &'static str>;
}

pub mod local_game_manager;

//game_manager keeps track of games progress and player roles (data)
//game_handler handles replies and prompts. Also decides which prompts to give out (logic)
