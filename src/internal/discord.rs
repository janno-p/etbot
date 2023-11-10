use serenity::{
    all::Message,
    builder::{CreateEmbed, CreateMessage},
    client::ClientBuilder,
    framework::{
        standard::{macros::hook, CommandError},
        StandardFramework,
    },
    http::HttpBuilder,
    model::{prelude::ChannelId, Color},
    prelude::{Context, GatewayIntents},
};
use tracing::{error, instrument};

use crate::commands::{GENERAL_GROUP, HELP, POTATOGAME_GROUP};

use super::handler::Handler;
use super::{bot::Bot, settings::Settings};

#[hook]
#[instrument]
async fn before_hook(ctx: &Context, msg: &Message, _: &str) -> bool {
    let data = ctx.data.read().await;
    let bot = data.get::<Bot>().unwrap();
    msg.channel_id == bot.potato_channel_id
}

#[hook]
#[instrument]
async fn after_hook(_: &Context, _: &Message, cmd_name: &str, err: Result<(), CommandError>) {
    if let Err(why) = err {
        error!("Error in {}: {:?}", cmd_name, why);
    }
}

#[instrument]
pub async fn start_client(bot: Bot, handler: Handler, settings: &Settings) {
    let framework = StandardFramework::new()
        .help(&HELP)
        .group(&GENERAL_GROUP)
        .group(&POTATOGAME_GROUP)
        .after(after_hook)
        .before(before_hook);

    framework.configure(|c| c.prefix("!").on_mention(None));

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut http = HttpBuilder::new(&settings.discord.token);
    if let Some(proxy) = &settings.discord.proxy {
        http = http.proxy(proxy);
    }

    let mut client = ClientBuilder::new_with_http(http.build(), intents)
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
        error!("An error occurred while running the client: {:?}", why);
    }
}

#[instrument]
pub async fn success_message(ctx: &Context, channel_id: &ChannelId, message: String) {
    let embed = CreateEmbed::new()
        .description(message)
        .color(Color::DARK_GREEN);

    let builder = CreateMessage::new().embed(embed);

    if let Err(why) = channel_id.send_message(ctx, builder).await {
        error!("Error sending message: {why:?}");
    }
}

#[instrument]
pub async fn failure_message(ctx: &Context, channel_id: &ChannelId, message: String) {
    let embed = CreateEmbed::new().description(message).color(Color::RED);

    let builder = CreateMessage::new().embed(embed);

    if let Err(why) = channel_id.send_message(ctx, builder).await {
        error!("Error sending message: {why:?}");
    }
}
