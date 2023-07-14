use dotenv::dotenv;
use std::error::Error;
use that_mafia_game::start_mafia_bot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    start_mafia_bot().await
}
