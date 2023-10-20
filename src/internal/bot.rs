use serenity::{model::prelude::ChannelId, prelude::TypeMapKey};

#[derive(Debug)]
pub struct Bot {
    pub database: sqlx::SqlitePool,
    pub potato_channel_id: ChannelId,
    pub potato_amount: i64,
}

impl Bot {
    pub fn new(
        database: sqlx::SqlitePool,
        potato_channel_id: ChannelId,
        potato_amount: i64,
    ) -> Self {
        Self {
            database,
            potato_channel_id,
            potato_amount,
        }
    }
}

impl TypeMapKey for Bot {
    type Value = Bot;
}
