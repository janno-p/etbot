use std::collections::BTreeMap;

use chrono::Utc;
use serenity::model::prelude::UserId;
use sqlx::{Pool, Sqlite};
use tracing::instrument;

use super::{model::Player, settings::Settings};

#[instrument]
pub async fn init(settings: &Settings) -> Pool<Sqlite> {
    sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions::new()
                .filename(&settings.database.filename)
                .create_if_missing(true),
        )
        .await
        .expect("Could not connect to database")
}

#[instrument]
pub async fn migrate(database: &Pool<Sqlite>) {
    sqlx::migrate!("./migrations")
        .run(database)
        .await
        .expect("Could not run database migrations");
}

#[instrument]
pub async fn find_player(user_id: &String, database: &Pool<Sqlite>) -> Option<Player> {
    sqlx::query_as!(
        Player,
        "SELECT discord_user_id, balance, last_feed_ts, version FROM players WHERE discord_user_id = ?",
        user_id
    )
    .fetch_optional(database)
    .await
    .unwrap_or(None)
}

#[instrument]
pub async fn find_unfeeded_players(ts: i64, database: &Pool<Sqlite>) -> Vec<String> {
    sqlx::query_scalar!(
        "SELECT discord_user_id FROM players WHERE last_feed_ts < ?",
        ts
    )
    .fetch_all(database)
    .await
    .unwrap_or(vec![])
}

#[instrument]
pub async fn create_player(user_id: &String, database: &Pool<Sqlite>) -> Option<Player> {
    let player = Player {
        discord_user_id: user_id.to_string(),
        balance: 5000,
        last_feed_ts: Utc::now().timestamp(),
        version: 1,
    };

    sqlx::query!(
        "INSERT INTO players (discord_user_id, balance, last_feed_ts, version) VALUES (?, ?, ?, ?)",
        player.discord_user_id,
        player.balance,
        player.last_feed_ts,
        player.version
    )
    .execute(database)
    .await
    .map_or(None, |_| Some(player))
}

#[instrument]
pub async fn update_player(player: &mut Player, database: &Pool<Sqlite>) -> bool {
    let current_version = player.version;
    player.version += 1;

    sqlx::query!(
        "UPDATE players SET balance = ?, last_feed_ts = ?, version = ? WHERE discord_user_id = ? AND version = ?",
        player.balance,
        player.last_feed_ts,
        player.version,
        player.discord_user_id,
        current_version
    )
    .execute(database)
    .await
    .ok()
    .map_or(false, |result| result.rows_affected() > 0)
}

#[instrument]
pub async fn load_leaderboard(database: &Pool<Sqlite>) -> Vec<(UserId, i64, usize, usize)> {
    let grouped_by_balance: BTreeMap<i64, Vec<UserId>> = sqlx::query_as!(
        Player,
        "SELECT discord_user_id, balance, last_feed_ts, version FROM players ORDER BY balance DESC"
    )
    .fetch_all(database)
    .await
    .unwrap_or_else(|_| Vec::new())
    .iter()
    .map(|p| {
        (
            UserId::new(p.discord_user_id.parse::<u64>().unwrap()),
            p.balance,
        )
    })
    .fold(BTreeMap::new(), |mut acc, (user_id, balance)| {
        acc.entry(balance).or_default().push(user_id);
        acc
    });
    grouped_by_balance
        .iter()
        .rev()
        .fold(Vec::new(), |mut acc: Vec<_>, (balance, user_ids)| {
            let min_pos = acc.len() + 1;
            let max_pos = min_pos + user_ids.len() - 1;
            acc.extend(
                user_ids
                    .iter()
                    .map(|user_id| (*user_id, *balance, min_pos, max_pos)),
            );
            acc
        })
}
