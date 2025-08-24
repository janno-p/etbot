use poise::serenity_prelude as serenity;

use crate::internal::{
    data::{
        Context,
        Error,
    },
    database,
    discord,
    shared,
};

/// Gives potatoes to another user.
/// 
/// Usage: `!give <amount> @<mention>`
/// 
/// Example: `!give 1500 @jaxx`
#[poise::command(
    broadcast_typing,
    category = "Potato Game",
    prefix_command,
)]
pub async fn give(
    ctx: Context<'_>,
    #[description = "The amount you want to give to another user"] amount: i64,
    #[description = "User to give your :potato: to"] user: serenity::User,
) -> Result<(), Error> {
    let user_name = user.global_name.unwrap_or(user.name);

    if amount < 1 {
        discord::failure_message(&ctx, "Kinkida saab minimaalselt 1 :potato:.").await;
        return Ok(());
    }

    if ctx.author().id == user.id {
        discord::failure_message(&ctx, "Enesearmastaja ei saa ühtegi :potato:.").await;
        return Ok(());
    }

    if user.bot {
        discord::failure_message(&ctx, format!("Bot {} on kasiinosõltlaste nimekirjas.", user_name)).await;
        return Ok(());
    }

    let mut sending_user = match database::find_player(&ctx.author().id.to_string(), &ctx.data().database)
        .await
    {
        Some(player) => player,
        None => {
            shared::create_new_player(&ctx, &ctx.author().id, &ctx.data().database).await?
        }
    };

    let mut receiving_user = match database::find_player(&user.id.to_string(), &ctx.data().database).await
    {
        Some(player) => player,
        None => shared::create_new_player(&ctx, &user.id, &ctx.data().database).await?,
    };

    if sending_user.balance < amount {
        discord::failure_message(&ctx, format!("{} Sul pole kinkimiseks piisavalt :potato:.", serenity::Mention::from(ctx.author().id))).await;
        return Ok(());
    }

    sending_user.balance -= amount;
    receiving_user.balance += amount;

    let tx = ctx.data().database.begin().await?;

    if database::update_player(&mut sending_user, &ctx.data().database).await
        && database::update_player(&mut receiving_user, &ctx.data().database).await
    {
        tx.commit().await?;
        discord::success_message(&ctx, format!("{} kinkis kasutajale {} {} :potato:.", serenity::Mention::from(ctx.author().id), serenity::Mention::from(user.id), amount)).await;
    }

    Ok(())
}
