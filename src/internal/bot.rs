use serenity::prelude::TypeMapKey;

pub struct Bot {
    #[allow(unused)]
    pub database: sqlx::SqlitePool,
}

impl Bot {
    pub fn new(database: sqlx::SqlitePool) -> Self {
        Self { database }
    }
}

impl TypeMapKey for Bot {
    type Value = Bot;
}
