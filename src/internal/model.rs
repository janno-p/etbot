#[derive(Clone, Debug)]
pub struct Player {
    pub discord_user_id: String,
    pub balance: i64,
    pub last_feed_ts: i64,
    pub version: i64,
}
