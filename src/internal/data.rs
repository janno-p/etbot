use serenity::all::ChannelId;

use crate::internal::feeder::Feeder;

#[derive(Debug)]
pub struct Data {
    pub database: sqlx::SqlitePool,
    pub potato_channel_id: ChannelId,
    pub zero_points_emoji: String,
    pub feeder: Feeder,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

impl Data {
    pub fn new(
        database: sqlx::SqlitePool,
        potato_channel_id: ChannelId,
        potato_amount: i64,
        zero_points_emoji: String,
    ) -> Self {
        Self {
            database: database.clone(),
            potato_channel_id,
            zero_points_emoji,
            feeder: Feeder::new(potato_channel_id, potato_amount, database),
        }
    }
}
