use poise::serenity_prelude as serenity;
use tracing::error;

use crate::internal::{
    data::{Context, Error},
    database,
};

/// Displays leaderboard.
#[poise::command(
    prefix_command,
    aliases("lb"),
    broadcast_typing,
    category = "Potato Game",
)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<(), Error> {
    let leaderboard = database::load_leaderboard(&ctx.data().database).await;

    let mut display_names = Vec::new();
    for &(user_id, _, _, _) in leaderboard.iter() {
        let user = user_id.to_user(ctx).await.unwrap();
        let display_name = user.global_name.unwrap_or(user.name);
        display_names.push(display_name);
    }

    let display_names = display_names;

    let embed = leaderboard.iter().enumerate().fold(
        serenity::CreateEmbed::new()
            .title(":potato: Leaderboard")
            .color(serenity::Color::DARK_GREEN),
        |embed, (i, (_, balance, from, to))| {
            let suffix = if from == to {
                "".to_string()
            } else {
                format!("-{}.", to)
            };
            let display_name = display_names.get(i).unwrap();
            let emoji = match (from, balance) {
                (1, _) => ":first_place:",
                (2, _) => ":second_place:",
                (3, _) => ":third_place:",
                (_, 0) => &ctx.data().zero_points_emoji,
                _ => "",
            };
            embed.field(
                format!("#{}.{} {}", from, suffix, display_name),
                format!("{} {}", emoji, balance),
                false,
            )
        },
    );

    let reply = poise::CreateReply::default()
        .embed(embed);

    if let Err(why) = ctx.send(reply).await {
        error!("Error sending message: {why:?}");
    }

    Ok(())
}
