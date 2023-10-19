use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::{Context, Mentionable},
    utils::parse_user_mention,
};

use crate::{
    commands::shared,
    internal::{bot::Bot, database, discord},
};

#[command]
#[description("Gives potatoes to another user.")]
#[usage("<amount> @<mention>")]
#[example("1500 @jaxx")]
pub async fn give(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let _ = msg.channel_id.start_typing(&ctx.http);

    let amount = args.single::<i64>()?;
    let mention = args.single::<String>()?;
    let user_id = parse_user_mention(mention.as_str()).unwrap();
    let user = user_id.to_user(&ctx).await?;
    let user_name = user.global_name.unwrap_or(user.name);

    if amount < 1 {
        discord::failure_message(
            ctx,
            &msg.channel_id,
            "Kinkida saab minimaalselt 1 :potato:.".to_string(),
        )
        .await;
        return Ok(());
    }

    if msg.author.id == user_id {
        discord::failure_message(
            ctx,
            &msg.channel_id,
            "Enesearmastaja ei saa ühtegi :potato:.".to_string(),
        )
        .await;
        return Ok(());
    }

    if user.bot {
        discord::failure_message(
            ctx,
            &msg.channel_id,
            format!("Bot {} on kasiinosõltlaste nimekirjas.", user_name),
        )
        .await;
        return Ok(());
    }

    let data = ctx.data.read().await;
    let bot = data.get::<Bot>().unwrap();

    let mut sending_user = match database::find_player(&msg.author.id.to_string(), &bot.database)
        .await
    {
        Some(player) => player,
        None => {
            shared::create_new_player(ctx, &msg.author.id, &msg.channel_id, &bot.database).await?
        }
    };

    let mut receiving_user = match database::find_player(&user_id.to_string(), &bot.database).await
    {
        Some(player) => player,
        None => shared::create_new_player(ctx, &user_id, &msg.channel_id, &bot.database).await?,
    };

    if sending_user.balance < amount {
        discord::failure_message(
            ctx,
            &msg.channel_id,
            format!(
                "{} Sul pole kinkimiseks piisavalt :potato:.",
                msg.author.id.mention()
            ),
        )
        .await;
        return Ok(());
    }

    sending_user.balance -= amount;
    receiving_user.balance += amount;

    let tx = bot.database.begin().await?;

    if database::update_player(&mut sending_user, &bot.database).await
        && database::update_player(&mut receiving_user, &bot.database).await
    {
        tx.commit().await?;
        discord::success_message(
            ctx,
            &msg.channel_id,
            format!(
                "{} kinkis kasutajale {} {} :potato:.",
                msg.author.id.mention(),
                user_id.mention(),
                amount
            ),
        )
        .await;
    }

    Ok(())
}
