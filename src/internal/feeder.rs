use chrono::{Datelike, Days, Local, NaiveTime, TimeZone, Weekday};
use poise::serenity_prelude as serenity;
use sqlx::{Pool, Sqlite};
use std::sync::Mutex;
use tracing::{error, info, instrument};

use crate::internal::database;

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
            return
        }

        *is_running = true;

        let channel_id = self.channel_id;
        let amount = self.amount;
        let database = self.database.clone();

        tokio::spawn(async move {
            let message = serenity::CreateMessage::new()
                .content("Oled valmis, Jaanus? Sõidame! :oncoming_automobile:");

            if let Err(why) = channel_id.send_message(&ctx.http, message).await {
                error!("Error sending message: {why:?}");
            }

            let mut interval_timer = tokio::time::interval(duration_str::parse("30s").unwrap());

            loop {
                interval_timer.tick().await;

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

                for user_id in database::find_unfeeded_players(dt.timestamp(), &database).await {
                    info!("Feeding potatoes to user {} ...", user_id);
                    loop {
                        let mut player = database::find_player(&user_id, &database).await.unwrap();
                        player.balance += amount;
                        player.last_feed_ts = now.timestamp();
                        if database::update_player(&mut player, &database).await {
                            let mention = serenity::Mention::from(serenity::UserId::new(user_id.parse::<u64>().unwrap()));
                            if let Err(why) = channel_id
                                .send_message(
                                    &ctx,
                                    serenity::CreateMessage::new().content(format!(
                                        "{} kartulisalve lisati {} :potato:.",
                                        mention, amount
                                    )),
                                )
                                .await
                            {
                                error!("Error sending message: {why:?}");
                            }
                            break;
                        }
                    }
                    info!("User {} has much more potatoes now", user_id);
                }
            }
        });
    }
}
