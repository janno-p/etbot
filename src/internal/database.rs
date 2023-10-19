use chrono::Utc;
use serenity::model::prelude::UserId;
use sqlx::{Pool, Sqlite};

use super::{model::Player, settings::Settings};

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

pub async fn migrate(database: &Pool<Sqlite>) {
    sqlx::migrate!("./migrations")
        .run(database)
        .await
        .expect("Could not run database migrations");
}

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

pub async fn find_unfeeded_players(ts: i64, database: &Pool<Sqlite>) -> Vec<String> {
    sqlx::query_scalar!(
        "SELECT discord_user_id FROM players WHERE last_feed_ts < ?",
        ts
    )
    .fetch_all(database)
    .await
    .unwrap_or(vec![])
}

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

pub async fn load_leaderboard(database: &Pool<Sqlite>) -> Vec<(UserId, i64, usize, usize)> {
    sqlx::query_as!(
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
    .enumerate()
    .fold(
        (vec![], 0),
        |(mut acc, pos): (Vec<(UserId, i64, usize, usize)>, usize), (i, (user_id, balance))| {
            match acc.last() {
                Some(&e) if e.1 == balance => {
                    acc.push((user_id, balance, pos, pos));
                    (acc, pos)
                }
                Some(_) => {
                    for n in pos..(i + 1) {
                        if let Some(q) = acc.get_mut(n) {
                            q.3 = i;
                        }
                    }
                    acc.push((user_id, balance, i + 1, i + 1));
                    (acc, i + 1)
                }
                None => {
                    acc.push((user_id, balance, 1, 1));
                    (acc, 1)
                }
            }
        },
    )
    .0
}
