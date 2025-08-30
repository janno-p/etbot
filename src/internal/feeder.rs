use chrono::{Datelike, Days, Local, NaiveTime, TimeZone, Weekday};
use poise::serenity_prelude as serenity;
use sqlx::{Pool, Sqlite};
use std::sync::Mutex;
use tracing::{error, info, instrument, warn};

use crate::database::players::{find_unfeeded_players, remove_player, update_player, IdleStatus};

#[derive(Debug)]
pub struct Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Feeder {
    channel_id: serenity::ChannelId,
    amount: i64,
    database: Pool<Sqlite>,
    is_running: Mutex<bool>,
}

impl Feeder {
    pub fn new(channel_id: serenity::ChannelId, amount: i64, database: Pool<Sqlite>) -> Self {
        Feeder {
            channel_id,
            amount,
            database,
            is_running: Mutex::new(false),
        }
    }

    #[instrument]
    pub fn start(&self, ctx: serenity::Context) {
        let mut is_running = self.is_running.lock().unwrap();
        if *is_running {
            return;
        }

        *is_running = true;

        let channel_id = self.channel_id;
        let amount = self.amount;
        let database = self.database.clone();

        tokio::spawn(async move {
            let message = serenity::CreateMessage::new()
                .content("@everyone Kas olete valmis? Sõidame! :oncoming_automobile:");

            if let Err(why) = channel_id.send_message(&ctx.http, message).await {
                error!("Error sending message: {why:?}");
            }

            let mut interval_timer = tokio::time::interval(duration_str::parse("30s").unwrap());

            loop {
                interval_timer.tick().await;

                let _ = do_feeding(&ctx, &database, channel_id, amount).await;
            }
        });
    }
}

async fn do_feeding(
    ctx: &serenity::Context,
    database: &Pool<Sqlite>,
    channel_id: serenity::ChannelId,
    amount: i64,
) -> Result<(), Error> {
    info!("Checking for the time to feed potatoes ...");

    let _ = channel_id.start_typing(&ctx.http);

    let now = Local::now();

    let mut last_friday = now.date_naive();
    while last_friday.weekday() != Weekday::Fri {
        last_friday = last_friday.checked_sub_days(Days::new(1)).unwrap();
    }

    let dt = Local
        .from_local_datetime(&last_friday.and_time(NaiveTime::MIN))
        .unwrap();

    let tx = database.begin().await.map_err(|_| Error {})?;

    let mut messages = Vec::<serenity::CreateMessage>::new();
    let mut charity_sum = 0i64;

    let all_players = find_unfeeded_players(dt.timestamp(), database).await;
    let (active_players, idle_players) =
        all_players
            .iter()
            .fold((vec![], vec![]), |(mut a1, mut a2), x| {
                if let IdleStatus::Active = x.idle_status(now.timestamp()) {
                    a1.push(x);
                } else {
                    a2.push(x);
                }
                (a1, a2)
            });

    for p in idle_players {
        let mut player = p.clone();
        if player.balance < 1 {
            while !remove_player(&mut player, database).await {
                warn!("Could not remove player {}", player.discord_user_id);
            }
            let mention = serenity::Mention::from(serenity::UserId::new(
                player.discord_user_id.parse::<u64>().unwrap(),
            ));
            messages.push(
                serenity::CreateMessage::new()
                    .content(format!("{} visati kartulikasiinost välja.", mention)),
            );
            continue;
        }
        let charity = player.charity_amount(now.timestamp());
        info!(
            "Taking {} potatoes from user {} for charity ...",
            charity, player.discord_user_id
        );
        charity_sum += charity;
        player.balance -= charity;
        player.last_feed_ts = now.timestamp();
        while !update_player(&mut player, database).await {
            warn!("Could not update player {}", player.discord_user_id);
        }
        let mention = serenity::Mention::from(serenity::UserId::new(
            player.discord_user_id.parse::<u64>().unwrap(),
        ));
        messages.push(serenity::CreateMessage::new().content(format!(
            "{} kartulisalvest võeti {} :potato: teistele jagamiseks.",
            mention, charity
        )));
    }

    let num_active_players = active_players.len() as i64;
    let total_amount: i64 = match num_active_players {
        0 => charity_sum,
        _ => amount + charity_sum / num_active_players,
    };

    for p in active_players {
        let mut player = p.clone();
        info!("Feeding potatoes to user {} ...", player.discord_user_id);
        player.balance += total_amount;
        player.last_feed_ts = now.timestamp();
        while !update_player(&mut player, database).await {
            warn!("Could not update player {}", player.discord_user_id);
        }
        let mention = serenity::Mention::from(serenity::UserId::new(
            player.discord_user_id.parse::<u64>().unwrap(),
        ));
        messages.push(serenity::CreateMessage::new().content(format!(
            "{} kartulisalve lisati {} :potato:.",
            mention, total_amount
        )));
        info!("User {} has much more potatoes now", player.discord_user_id);
    }

    tx.commit().await.map_err(|_| Error {})?;

    for message in messages {
        if let Err(why) = channel_id.send_message(&ctx, message).await {
            error!("Error sending message: {why:?}");
        }
    }

    Ok(())
}
