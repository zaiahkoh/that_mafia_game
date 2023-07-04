use teloxide::types::ChatId;

#[derive(Clone)]
pub struct Player {
    pub chat_id: ChatId,
    pub username: String,
    pub role: Role,
    pub is_alive: bool,
    pub is_connected: bool,
}

#[derive(Copy, Clone, Debug)]
pub enum Role {
    Mafia,
    Civilian,
}
