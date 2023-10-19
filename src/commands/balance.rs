use serenity::{
    framework::standard::{macros::command, CommandResult},
    model::prelude::Message,
    prelude::{Context, Mentionable},
};

use crate::{
    commands::shared,
    internal::{bot::Bot, database, discord},
};

#[command]
#[aliases("$$")]
#[description("Shows how many :potato: you've got.")]
pub async fn balance(ctx: &Context, msg: &Message) -> CommandResult {
    let _ = msg.channel_id.start_typing(&ctx.http);

    let data = ctx.data.read().await;
    let bot = data.get::<Bot>().unwrap();

    let user_id = msg.author.id.to_string();
    let user_mention = msg.author.mention();

    let player = database::find_player(&user_id, &bot.database).await;

    match player {
        Some(player) => {
            let message = format!("{} kontol on {} :potato:.", user_mention, player.balance);
            discord::success_message(ctx, &msg.channel_id, message).await;
        }
        None => {
            shared::create_new_player(ctx, &msg.author.id, &msg.channel_id, &bot.database).await?;
        }
    }

    Ok(())
}
