use std::str::FromStr;

use serenity::{
    framework::standard::{macros::command, Args, CommandResult},
    model::prelude::Message,
    prelude::Context,
};

#[derive(Debug)]
enum BetAmount {
    Specific(u64),
    Percentage(u8),
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
            val if val.ends_with('%') => val
                .trim_end_matches('%')
                .parse::<u8>()
                .map_or(Err(ParseBetAmountError), |x| Ok(BetAmount::Percentage(x))),
            val => val
                .parse::<u64>()
                .map_or(Err(ParseBetAmountError), |x| Ok(BetAmount::Specific(x))),
        }
    }
}

#[derive(Debug)]
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
    msg.channel_id.start_typing(&ctx.http).ok();

    match (args.single::<BetAmount>(), args.single::<CoinSide>()) {
        (Ok(bet_amount), Ok(coin_side)) => {
            msg.reply(
                ctx,
                format!("Requested 'flip': {:?}; {:?}", bet_amount, coin_side),
            )
            .await?;
        }
        _ => {
            msg.reply(ctx, "Ei saa aru, mida sa teha tahad!").await?;
        }
    }

    // let mut data = ctx.data.read().await;
    // let bot = data.get::<Bot>().unwrap();

    Ok(())
}
