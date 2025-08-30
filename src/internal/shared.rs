use serenity::{all::UserId, prelude::Mentionable};

use sqlx::{Pool, Sqlite};

use crate::database::players::{create_player, Player};

use crate::internal::{data::Context, discord, errors::PotatoGameError};

pub async fn create_new_player(
    ctx: &Context<'_>,
    user_id: &UserId,
    database: &Pool<Sqlite>,
) -> Result<Player, PotatoGameError> {
    match create_player(&user_id.to_string(), database).await {
        Some(player) => {
            discord::success_message(ctx, format!("{} pole varasemalt kartulikasiinos mänginud, viskasin seemneks kontole 5000 :potato:.", user_id.mention())).await;
            Ok(player)
        }
        None => Err(PotatoGameError::ConcurrencyError),
    }
}
