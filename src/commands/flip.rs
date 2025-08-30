use chrono::Utc;
use poise::serenity_prelude as serenity;
use rand::Rng;
use std::str::FromStr;

use crate::database::players::{find_player, update_player, Player};
use crate::internal::data::{Context, Error};
use crate::internal::discord;
use crate::internal::errors::PotatoGameError;
use crate::internal::shared;

#[derive(Debug)]
enum BetAmount {
    Specific(i64),
    Percentage(i8),
    Half,
    All,
    Some,
}

#[derive(Debug)]
struct ParseBetAmountError;

impl FromStr for BetAmount {
    type Err = ParseBetAmountError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "all" => Ok(BetAmount::All),
            "half" => Ok(BetAmount::Half),
            "some" => Ok(BetAmount::Some),
            val if val.ends_with('%') => {
                val.trim_end_matches('%')
                    .parse::<i8>()
                    .ok()
                    .map_or(Err(ParseBetAmountError), |v| match v {
                        x if (0..=100).contains(&x) => Ok(BetAmount::Percentage(x)),
                        _ => Err(ParseBetAmountError),
                    })
            }
            val => val
                .parse::<i64>()
                .ok()
                .map_or(Err(ParseBetAmountError), |v| match v {
                    x if x >= 0 => Ok(BetAmount::Specific(v)),
                    _ => Err(ParseBetAmountError),
                }),
        }
    }
}

#[derive(Debug, PartialEq, poise::ChoiceParameter)]
enum CoinSide {
    #[name = "h"]
    #[name = "heads"]
    Heads,
    #[name = "t"]
    #[name = "tails"]
    Tails,
}

/// Flip a coin - game for fun.
///
/// Usage: `all|half|some|<amount>[%] h|heads|t|tails`
///
/// Example: `!flip all tails`
/// Example: `!flip 3000 heads`
/// Example: `!flip 33% t`
#[poise::command(broadcast_typing, category = "Potato Game", prefix_command)]
pub async fn flip(
    ctx: Context<'_>,
    #[description = "The amount you want to bet on"] bet_amount_str: String,
    #[description = "The coin side you want to choose"] coin_side: CoinSide,
) -> Result<(), Error> {
    match BetAmount::from_str(&bet_amount_str) {
        Ok(bet_amount) => {
            process(ctx, &bet_amount, &coin_side).await?;
        }
        _ => {
            let reply = poise::CreateReply::default().content("Ei saa aru, mida sa teha tahad!");

            ctx.send(reply).await?;
        }
    }

    Ok(())
}

async fn process(
    ctx: Context<'_>,
    bet_amount: &BetAmount,
    coin_side: &CoinSide,
) -> Result<(), Error> {
    let user_id = ctx.author().id.to_string();
    let user_mention = serenity::Mention::from(ctx.author().id);

    let player = find_player(&user_id, &ctx.data().database).await;

    let mut player = match player {
        Some(player) => player,
        None => shared::create_new_player(&ctx, &ctx.author().id, &ctx.data().database).await?,
    };

    let amount = calculate_amount(bet_amount, &player);

    if amount < 2 {
        discord::failure_message(
            &ctx,
            format!("{} Minimaalne panus on 2 :potato:.", user_mention),
        )
        .await;
        return Ok(());
    }

    if amount > player.balance {
        discord::failure_message(
            &ctx,
            format!(
                "{} Sul pole panuse tegemiseks piisavalt :potato:.",
                user_mention
            ),
        )
        .await;
        return Ok(());
    }

    let toss_result = if rand::rng().random::<bool>() {
        CoinSide::Heads
    } else {
        CoinSide::Tails
    };

    let is_win = toss_result == *coin_side;
    if is_win {
        player.balance += amount;
    } else {
        player.balance -= amount;
    }

    player.idle_since_ts = Utc::now().timestamp();

    if !update_player(&mut player, &ctx.data().database).await {
        return Err(Box::new(PotatoGameError::ConcurrencyError));
    }

    if !is_win {
        let message = if let BetAmount::Specific(_) = bet_amount {
            format!(
                "{} Seekord läks halvasti, järgmine kord on ehk rohkem õnne :cry:.",
                user_mention
            )
        } else {
            format!(
                "{} Seekord läks halvasti, oled {} :potato: võrra vaesem. Järgmine kord on ehk rohkem õnne :cry:.",
                user_mention, amount
            )
        };
        discord::failure_message(&ctx, message).await;
        return Ok(());
    }

    discord::success_message(
        &ctx,
        format!(
            "{} Palju õnne! Võitsid {} :potato:",
            user_mention,
            (amount * 2)
        ),
    )
    .await;

    Ok(())
}

fn calculate_amount(bet_amount: &BetAmount, player: &Player) -> i64 {
    match bet_amount {
        BetAmount::All => player.balance,
        BetAmount::Half => player.balance / 2,
        BetAmount::Some => rand::rng().random_range(2i64..=player.balance),
        BetAmount::Specific(v) => *v,
        BetAmount::Percentage(v) => (*v) as i64 * player.balance / 100i64,
    }
}
