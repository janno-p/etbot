use serenity::{
    async_trait,
    model::prelude::{ChannelId, Ready},
    prelude::{Context, EventHandler},
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _ready: Ready) {
        let channel_id = ChannelId(474886383425552384);

        channel_id
            .send_message(&ctx.http, |message| {
                message.content("Oled valmis, Jaanus? Sõidame!")
            })
            .await
            .ok();
    }
}
