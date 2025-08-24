use poise::serenity_prelude as serenity;
use tracing::{error, info, instrument};

use super::data::{Context, Data, Error};
use super::settings::Settings;

#[instrument(skip(_framework))]
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::Ready { data_about_bot, .. } = event {
        info!("Logged in as {}", data_about_bot.user.name);
        data.feeder.start(ctx.clone());
    }
    Ok(())
}

#[instrument]
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[instrument]
pub async fn start_client(data: Data, settings: &Settings) {
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            command_check: Some(|ctx: Context| {
                Box::pin(async move {
                    Ok(ctx.channel_id() == ctx.data().potato_channel_id)
                })
            }),
            commands: vec![
                crate::commands::balance::balance(),
                crate::commands::flip::flip(),
                crate::commands::give::give(),
                crate::commands::help::help(),
                crate::commands::leaderboard::leaderboard(),
                crate::commands::ping::ping(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            on_error: |error| Box::pin(on_error(error)),
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".into()),
                mention_as_prefix: false,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data)
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut http = serenity::HttpBuilder::new(&settings.discord.token);
    if let Some(proxy) = &settings.discord.proxy {
        http = http.proxy(proxy);
    }

    let mut client = serenity::ClientBuilder::new_with_http(http.build(), intents)
        .framework(framework)
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

pub async fn success_message(ctx: &Context<'_>, message: impl Into<String>) {
    let embed = serenity::CreateEmbed::new()
        .description(message)
        .color(serenity::Color::DARK_GREEN);

    let reply = poise::CreateReply::default().embed(embed);

    if let Err(why) = ctx.send(reply).await {
        error!("Error sending message: {why:?}");
    }
}

pub async fn failure_message(ctx: &Context<'_>, message: impl Into<String>) {
    let embed = serenity::CreateEmbed::new()
        .description(message)
        .color(serenity::Color::RED);

    let reply = poise::CreateReply::default().embed(embed);

    if let Err(why) = ctx.send(reply).await {
        error!("Error sending message: {why:?}");
    }
}
