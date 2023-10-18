use std::str::FromStr;

use rand::Rng;
use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::{Context, Mentionable},
};

use crate::internal::{bot::Bot, database, discord, model::Player};

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

#[derive(Debug, PartialEq)]
enum CoinSide {
    Heads,
    Tails,
}

#[derive(Debug)]
struct ParseCoinSideError;

impl FromStr for CoinSide {
    type Err = ParseCoinSideError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "h" | "heads" => Ok(CoinSide::Heads),
            "t" | "tails" => Ok(CoinSide::Tails),
            _ => Err(ParseCoinSideError),
        }
    }
}

#[command]
#[description("Flip a coin - game for fun.")]
#[usage("all|half|some|<amount>[%] h|heads|t|tails")]
#[example("all tails")]
#[example("3000 heads")]
#[example("33% t")]
pub async fn flip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let _ = msg.channel_id.start_typing(&ctx.http);

    let data = ctx.data.read().await;
    let bot = data.get::<Bot>().unwrap();

    match (args.single::<BetAmount>(), args.single::<CoinSide>()) {
        (Ok(bet_amount), Ok(coin_side)) => {
            process(ctx, msg, bot, &bet_amount, &coin_side).await?;
        }
        _ => {
            msg.reply(ctx, "Ei saa aru, mida sa teha tahad!").await?;
        }
    }

    Ok(())
}

async fn process(
    ctx: &Context,
    msg: &Message,
    bot: &Bot,
    bet_amount: &BetAmount,
    coin_side: &CoinSide,
) -> CommandResult {
    let user_id = msg.author.id.to_string();
    let user_mention = msg.author.mention();

    let player = database::find_player(&user_id, &bot.database).await;

    let mut player = match player {
        Some(player) => player,
        None => match database::create_player(&user_id, &bot.database).await {
            Some(player) => {
                let message = format!("{} pole varasemalt kartulikasiinos mänginud, viskasin seemneks kontole 5000 :potato:.", user_mention);
                discord::success_message(ctx, &msg.channel_id, message).await;
                player
            }
            None => {
                let message = format!("UPS!! Proovi uuesti, {}!", user_mention);
                discord::failure_message(ctx, &msg.channel_id, message).await;
                return Ok(());
            }
        },
    };

    let amount = calculate_amount(bet_amount, &player);

    if amount < 2 {
        let message = format!("{} Minimaalne panus on 2 :potato:.", user_mention);
        discord::failure_message(ctx, &msg.channel_id, message).await;
        return Ok(());
    }

    if amount > player.balance {
        let message = format!(
            "{} Sul pole panuse tegemiseks piisavalt :potato:.",
            user_mention
        );
        discord::failure_message(ctx, &msg.channel_id, message).await;
        return Ok(());
    }

    let toss_result = if rand::thread_rng().gen::<bool>() {
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

    if !database::update_player(&mut player, &bot.database).await {
        let message = format!("UPS!! Proovi uuesti, {}!", user_mention);
        discord::failure_message(ctx, &msg.channel_id, message).await;
        return Ok(());
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
        discord::failure_message(ctx, &msg.channel_id, message).await;
        return Ok(());
    }

    let message = format!(
        "{} Palju õnne! Võitsid {} :potato:",
        user_mention,
        (amount * 2)
    );
    discord::success_message(ctx, &msg.channel_id, message).await;

    Ok(())
}

fn calculate_amount(bet_amount: &BetAmount, player: &Player) -> i64 {
    match bet_amount {
        BetAmount::All => player.balance,
        BetAmount::Half => player.balance / 2,
        BetAmount::Some => rand::thread_rng().gen_range(2i64..=player.balance),
        BetAmount::Specific(v) => *v,
        BetAmount::Percentage(v) => (*v) as i64 * player.balance / 100i64,
    }
}
