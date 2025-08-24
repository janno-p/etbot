use poise::serenity_prelude as serenity;

use crate::{
    internal::data::{
        Context,
        Error
    },
    internal::{
        database,
        discord
    },
    internal::shared,
};

/// Shows how many :potato: you've got.
#[poise::command(
    prefix_command,
    aliases("$$"),
    category = "Potato Game",
    broadcast_typing,
)]
pub async fn balance(ctx: Context<'_>) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();
    let user_mention = serenity::Mention::from(ctx.author().id);

    let player = database::find_player(&user_id, &ctx.data().database).await;

    match player {
        Some(player) => {
            discord::success_message(&ctx, format!("{} kontol on {} :potato:.", user_mention, player.balance)).await;
        }
        None => {
            shared::create_new_player(&ctx, &ctx.author().id, &ctx.data().database).await?;
        }
    }

    Ok(())
}
