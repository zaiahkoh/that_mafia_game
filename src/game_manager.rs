use teloxide::types::ChatId;

use crate::game::Game;

pub trait GameManager {
    // Gets the instantaneous lobby, if present, of a chat user.
    fn get_player_game(&mut self, chat_id: ChatId) -> Option<&mut Game>;

    // Adds game to the map
    fn add_game(&mut self, game: Game);

    fn update_game(&mut self, game: Game, chat_id: ChatId);

    // If the host quits, then a remaining player should be randomly chosen to be the new host
    fn quit_game(&mut self, chat_id: ChatId) -> Result<&Game, &'static str>;
}

pub mod local_game_manager;

//game_manager keeps track of games progress and player roles (data)
//game_handler handles replies and prompts. Also decides which prompts to give out (logic)
