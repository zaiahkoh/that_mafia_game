use teloxide::types::ChatId;

use crate::game_interface::Game;

pub trait GameManager<G>
where
    G: Game,
{
    /// Returns a mutable reference to the chat_id's game, if present
    fn get_player_game(&mut self, chat_id: ChatId) -> Option<&mut G>;

    // Adds game to the map
    fn add_game(&mut self, game: G);

    fn update_game(&mut self, game: G, chat_id: ChatId);

    // If the host quits, then a remaining player should be randomly chosen to be the new host
    fn quit_game(&mut self, chat_id: ChatId) -> Result<&mut G, &'static str>;
}

pub mod local_game_manager;

//game_manager keeps track of games progress and player roles (data)
//game_handler handles replies and prompts. Also decides which prompts to give out (logic)
