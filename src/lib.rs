use teloxide::types::ChatId;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    GameRunning {
        host: ChatId
    },
}

pub mod game;
pub mod lobby;
