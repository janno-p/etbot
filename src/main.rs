mod commands;
mod internal;

use dotenv::dotenv;
use internal::bot::Bot;
use internal::database;
use internal::discord;
use internal::handler::Handler;
use internal::settings::Settings;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let settings = Settings::new().expect("Could not load bot settings");
    println!("{:?}", settings);

    let database = database::init(&settings).await;
    database::migrate(&database).await;

    let bot = Bot::new(database);

    discord::start_client(bot, Handler, &settings).await;
}