use serenity::{
    builder::{CreateEmbed, CreateMessage},
    framework::standard::{macros::command, CommandResult},
    model::{prelude::Message, Color},
    prelude::Context,
};
use tracing::error;

use crate::internal::{bot::Bot, database};

#[command]
#[aliases("lb")]
#[description("Displays leaderboard.")]
pub async fn leaderboard(ctx: &Context, msg: &Message) -> CommandResult {
    let _ = msg.channel_id.start_typing(&ctx.http);

    let data = ctx.data.read().await;
    let bot = data.get::<Bot>().unwrap();

    let leaderboard = database::load_leaderboard(&bot.database).await;

    let mut display_names = Vec::new();
    for &(user_id, _, _, _) in leaderboard.iter() {
        let user = user_id.to_user(ctx).await.unwrap();
        let display_name = user.global_name.unwrap_or(user.name);
        display_names.push(display_name);
    }

    let display_names = display_names;

    let embed = leaderboard.iter().enumerate().fold(
        CreateEmbed::new()
            .title(":potato: Leaderboard")
            .color(Color::DARK_GREEN),
        |embed, (i, (_, balance, from, to))| {
            let suff = if from == to {
                "".to_string()
            } else {
                format!("-{}.", to)
            };
            let display_name = display_names.get(i).unwrap();
            let emoji = match (from, balance) {
                (1, _) => ":first_place:",
                (2, _) => ":second_place:",
                (3, _) => ":third_place:",
                (_, 0) => &bot.zero_points_emoji,
                _ => "",
            };
            embed.field(
                format!("#{}.{} {}", from, suff, display_name),
                format!("{} {}", emoji, balance),
                false,
            )
        },
    );

    let message = CreateMessage::new().embed(embed);

    if let Err(why) = msg.channel_id.send_message(ctx, message).await {
        error!("Error sending message: {why:?}");
    }

    Ok(())
}
