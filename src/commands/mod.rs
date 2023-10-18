use serenity::framework::standard::macros::group;

pub mod balance;
pub mod flip;
pub mod help;
pub mod ping;

use self::balance::*;
use self::flip::*;
use self::ping::*;

#[group]
#[commands(ping)]
struct General;

#[group]
#[commands(balance, flip)]
struct PotatoGame;
