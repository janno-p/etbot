use serenity::{
    framework::StandardFramework,
    model::prelude::ChannelId,
    prelude::{Context, GatewayIntents},
    utils::Color,
    Client,
};

use crate::commands::{help::HELP, GENERAL_GROUP, POTATOGAME_GROUP};

use super::handler::Handler;
use super::{bot::Bot, settings::Settings};

pub async fn start_client(bot: Bot, handler: Handler, settings: &Settings) {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!").on_mention(None))
        .help(&HELP)
        .group(&GENERAL_GROUP)
        .group(&POTATOGAME_GROUP);

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&settings.discord.token, intents)
        .event_handler(handler)
        .framework(framework)
        .type_map_insert::<Bot>(bot)
        .await
        .expect("Error creating client");

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

pub async fn success_message(ctx: &Context, channel_id: &ChannelId, message: String) {
    channel_id
        .send_message(ctx, |m| {
            m.add_embed(|e| e.description(message).color(Color::DARK_GREEN))
        })
        .await
        .ok();
}

pub async fn failure_message(ctx: &Context, channel_id: &ChannelId, message: String) {
    channel_id
        .send_message(ctx, |m| {
            m.add_embed(|e| e.description(message).color(Color::RED))
        })
        .await
        .ok();
}
