use serenity::{model::prelude::ChannelId, prelude::TypeMapKey};

pub struct Bot {
    pub database: sqlx::SqlitePool,
    pub potato_channel_id: ChannelId,
}

impl Bot {
    pub fn new(database: sqlx::SqlitePool, potato_channel_id: ChannelId) -> Self {
        Self {
            database,
            potato_channel_id,
        }
    }
}

impl TypeMapKey for Bot {
    type Value = Bot;
}
