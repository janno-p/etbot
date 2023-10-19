use serenity::{
    async_trait,
    builder::CreateMessage,
    model::prelude::Ready,
    prelude::{Context, EventHandler},
};

use super::bot::Bot;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let data = ctx.data.read().await;
        let bot = data.get::<Bot>().unwrap();

        let message =
            CreateMessage::new().content("Oled valmis, Jaanus? Sõidame! :oncoming_automobile:");

        if let Err(why) = bot.potato_channel_id.send_message(&ctx.http, message).await {
            println!("Error sending message: {why:?}");
        }
    }
}
