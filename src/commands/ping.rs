use crate::internal::data::{Context, Error};

#[poise::command(broadcast_typing, category = "General", prefix_command)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}
