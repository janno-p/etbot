mod commands;
mod internal;

use dotenv::dotenv;
use internal::bot::Bot;
use internal::database;
use internal::discord;
use internal::handler::Handler;
use internal::settings::Settings;
use serenity::model::prelude::ChannelId;
use tracing::instrument;

#[tokio::main]
#[instrument]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();
    dotenv::from_filename(".env.local").ok();

    let settings = Settings::new().expect("Could not load bot settings");

    let database = database::init(&settings).await;
    database::migrate(&database).await;

    let bot = Bot::new(
        database,
        ChannelId::new(settings.potato_feeder.channel_id),
        settings.potato_feeder.amount,
    );

    discord::start_client(bot, Handler, &settings).await;
}
