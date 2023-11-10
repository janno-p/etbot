use serenity::{
    async_trait,
    builder::CreateMessage,
    model::prelude::Ready,
    prelude::{Context, EventHandler},
};
use tracing::{error, instrument};

use super::bot::Bot;

#[derive(Debug)]
pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, ctx))]
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let data = ctx.data.read().await;
        let bot = data.get::<Bot>().unwrap();

        crate::commands::feeder::start_feeder(
            ctx.clone(),
            bot.potato_channel_id,
            bot.potato_amount,
            bot.database.clone(),
        )
        .await;

        let message =
            CreateMessage::new().content("Oled valmis, Jaanus? Sõidame! :oncoming_automobile:");

        if let Err(why) = bot.potato_channel_id.send_message(&ctx.http, message).await {
            error!("Error sending message: {why:?}");
        }
    }
}
