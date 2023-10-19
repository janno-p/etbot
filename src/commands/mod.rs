use serenity::framework::standard::macros::group;

mod errors;
mod shared;

pub mod feeder;

mod balance;
mod flip;
mod give;
mod help;
mod leaderboard;
mod ping;

use self::balance::*;
use self::flip::*;
use self::give::*;
use self::leaderboard::*;
use self::ping::*;

pub use self::help::HELP;

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(balance, flip, give, leaderboard)]
struct PotatoGame;
