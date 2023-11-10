use serenity::{model::prelude::ChannelId, prelude::TypeMapKey};

#[derive(Debug)]
pub struct Bot {
    pub database: sqlx::SqlitePool,
    pub potato_channel_id: ChannelId,
    pub potato_amount: i64,
    pub zero_points_emoji: String,
}

impl Bot {
    pub fn new(
        database: sqlx::SqlitePool,
        potato_channel_id: ChannelId,
        potato_amount: i64,
        zero_points_emoji: String,
    ) -> Self {
        Self {
            database,
            potato_channel_id,
            potato_amount,
            zero_points_emoji,
        }
    }
}

impl TypeMapKey for Bot {
    type Value = Bot;
}
