use chrono::Utc;
use poise::serenity_prelude as serenity;
use sqlx::{Pool, Sqlite};
use std::collections::BTreeMap;
use tracing::instrument;

#[derive(Clone, Debug)]
pub struct Player {
    pub discord_user_id: String,
    pub balance: i64,
    pub last_feed_ts: i64,
    pub idle_since_ts: i64,
    pub version: i64,
}

pub enum IdleStatus {
    Active,
    Away,
    Sleeping,
    Missing,
    Dead,
}

pub fn get_idle_status(idle_time: i64) -> IdleStatus {
    match idle_time {
        ..604_800 => IdleStatus::Active,
        604_800..1_209_600 => IdleStatus::Away,
        1_209_600..1_814_400 => IdleStatus::Sleeping,
        1_814_400..2_419_200 => IdleStatus::Missing,
        2_419_200.. => IdleStatus::Dead,
    }
}

impl Player {
    pub fn idle_status(&self, ts: i64) -> IdleStatus {
        get_idle_status(ts - self.idle_since_ts)
    }

    pub fn charity_amount(&self, ts: i64) -> i64 {
        match self.idle_status(ts) {
            IdleStatus::Active => 0,
            IdleStatus::Away => self.balance / 10,
            IdleStatus::Sleeping => self.balance / 6,
            IdleStatus::Missing => self.balance / 3,
            IdleStatus::Dead => self.balance,
        }
    }
}

#[instrument]
pub async fn find_player(user_id: &String, database: &Pool<Sqlite>) -> Option<Player> {
    sqlx::query_as!(
        Player,
        "SELECT discord_user_id, balance, last_feed_ts, idle_since_ts, version FROM players WHERE discord_user_id = ?",
        user_id
    )
    .fetch_optional(database)
    .await
    .unwrap_or(None)
}

#[instrument]
pub async fn find_unfeeded_players(ts: i64, database: &Pool<Sqlite>) -> Vec<Player> {
    sqlx::query_as!(
        Player,
        "SELECT discord_user_id, balance, last_feed_ts, idle_since_ts, version FROM players WHERE last_feed_ts < ?",
        ts
    )
    .fetch_all(database)
    .await
    .unwrap_or(vec![])
}

#[instrument]
pub async fn create_player(user_id: &String, database: &Pool<Sqlite>) -> Option<Player> {
    let current_ts = Utc::now().timestamp();

    let player = Player {
        discord_user_id: user_id.to_string(),
        balance: 5000,
        last_feed_ts: current_ts,
        idle_since_ts: current_ts,
        version: 1,
    };

    sqlx::query!(
        "INSERT INTO players (discord_user_id, balance, last_feed_ts, idle_since_ts, version) VALUES (?, ?, ?, ?, ?)",
        player.discord_user_id,
        player.balance,
        player.last_feed_ts,
        player.idle_since_ts,
        player.version
    )
    .execute(database)
    .await
    .map_or(None, |_| Some(player))
}

#[instrument]
pub async fn remove_player(player: &mut Player, database: &Pool<Sqlite>) -> bool {
    let current_version = player.version;
    player.version += 1;

    sqlx::query!(
        "DELETE FROM players WHERE discord_user_id = ? AND version = ?",
        player.discord_user_id,
        current_version
    )
    .execute(database)
    .await
    .ok()
    .map_or_else(|| false, |result| result.rows_affected() > 0)
}

#[instrument]
pub async fn update_player(player: &mut Player, database: &Pool<Sqlite>) -> bool {
    let current_version = player.version;
    player.version += 1;

    sqlx::query!(
        "UPDATE players SET balance = ?, last_feed_ts = ?, idle_since_ts = ?, version = ? WHERE discord_user_id = ? AND version = ?",
        player.balance,
        player.last_feed_ts,
        player.idle_since_ts,
        player.version,
        player.discord_user_id,
        current_version
    )
    .execute(database)
    .await
    .ok()
    .map_or_else(|| false, |result| result.rows_affected() > 0)
}

#[instrument]
pub async fn load_leaderboard(
    database: &Pool<Sqlite>,
) -> Vec<(serenity::UserId, i64, usize, usize, i64)> {
    let grouped_by_balance: BTreeMap<i64, Vec<(serenity::UserId, i64)>> = sqlx::query_as!(
        Player,
        "SELECT discord_user_id, balance, last_feed_ts, idle_since_ts, version FROM players ORDER BY balance DESC"
    )
    .fetch_all(database)
    .await
    .unwrap_or_else(|_| Vec::new())
    .iter()
    .map(|p| {
        (
            (serenity::UserId::new(p.discord_user_id.parse::<u64>().unwrap()), p.idle_since_ts),
            p.balance,
        )
    })
    .fold(BTreeMap::new(), |mut acc, (row, balance)| {
        acc.entry(balance).or_default().push(row);
        acc
    });
    grouped_by_balance
        .iter()
        .rev()
        .fold(Vec::new(), |mut acc: Vec<_>, (balance, users)| {
            let min_pos = acc.len() + 1;
            let max_pos = min_pos + users.len() - 1;
            acc.extend(
                users
                    .iter()
                    .map(|(user_id, idle_ts)| (*user_id, *balance, min_pos, max_pos, *idle_ts)),
            );
            acc
        })
}
