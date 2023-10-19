use serenity::{
    all::{ChannelId, UserId},
    prelude::{Context, Mentionable},
};
use sqlx::{Pool, Sqlite};

use crate::internal::{database, discord, model::Player};

use super::errors::PotatoGameError;

pub async fn create_new_player(
    ctx: &Context,
    user_id: &UserId,
    channel_id: &ChannelId,
    database: &Pool<Sqlite>,
) -> Result<Player, PotatoGameError> {
    match database::create_player(&user_id.to_string(), database).await {
        Some(player) => {
            let message = format!("{} pole varasemalt kartulikasiinos mänginud, viskasin seemneks kontole 5000 :potato:.", user_id.mention());
            discord::success_message(ctx, channel_id, message).await;
            Ok(player)
        }
        None => Err(PotatoGameError::ConcurrencyError),
    }
}
