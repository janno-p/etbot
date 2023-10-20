use chrono::{Datelike, Days, Local, NaiveTime, TimeZone, Weekday};
use serenity::{
    all::{ChannelId, UserId},
    builder::CreateMessage,
    prelude::{Context, Mentionable},
};
use sqlx::{Pool, Sqlite};
use tracing::{instrument, info, error};

use crate::internal::database;

#[instrument]
pub async fn start_feeder(
    ctx: Context,
    channel_id: ChannelId,
    amount: i64,
    database: Pool<Sqlite>,
) {
    tokio::spawn(async move {
        let mut interval_timer = tokio::time::interval(duration_str::parse("30s").unwrap());
        loop {
            interval_timer.tick().await;

            info!("Checking for the time to feed potatoes ...");

            channel_id.start_typing(&ctx.http);

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
                        let mention = UserId::new(user_id.parse::<u64>().unwrap()).mention();
                        if let Err(why) = channel_id
                            .send_message(
                                &ctx,
                                CreateMessage::new().content(format!(
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
