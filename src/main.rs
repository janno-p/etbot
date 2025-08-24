mod commands;
mod internal;

use dotenv::dotenv;
use internal::data::Data;
use internal::database;
use internal::discord;
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

    let data = Data::new(
        database,
        ChannelId::new(settings.potato_feeder.channel_id),
        settings.potato_feeder.amount,
        settings.potato_feeder.zero_points_emoji.clone(),
    );

    discord::start_client(data, &settings).await;
}
