use serenity::{
    builder::{CreateEmbed, CreateMessage},
    framework::StandardFramework,
    model::{prelude::ChannelId, Color},
    prelude::{Context, GatewayIntents},
    Client,
};

use crate::commands::{GENERAL_GROUP, HELP, POTATOGAME_GROUP};

use super::handler::Handler;
use super::{bot::Bot, settings::Settings};

pub async fn start_client(bot: Bot, handler: Handler, settings: &Settings) {
    let framework = StandardFramework::new()
        .help(&HELP)
        .group(&GENERAL_GROUP)
        .group(&POTATOGAME_GROUP);

    framework.configure(|c| c.prefix("!").on_mention(None));

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
        shard_manager.shutdown_all().await;
    });

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}

pub async fn success_message(ctx: &Context, channel_id: &ChannelId, message: String) {
    let embed = CreateEmbed::new()
        .description(message)
        .color(Color::DARK_GREEN);

    let builder = CreateMessage::new().embed(embed);

    if let Err(why) = channel_id.send_message(ctx, builder).await {
        println!("Error sending message: {why:?}");
    }
}

pub async fn failure_message(ctx: &Context, channel_id: &ChannelId, message: String) {
    let embed = CreateEmbed::new().description(message).color(Color::RED);

    let builder = CreateMessage::new().embed(embed);

    if let Err(why) = channel_id.send_message(ctx, builder).await {
        println!("Error sending message: {why:?}");
    }
}
