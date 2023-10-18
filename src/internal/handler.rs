use serenity::{
    async_trait,
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

        bot.potato_channel_id
            .send_message(&ctx.http, |message| {
                message.content("Oled valmis, Jaanus? Sõidame!")
            })
            .await
            .ok();
    }
}
