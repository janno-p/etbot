use std::error::Error;
use std::fmt::Display;

#[derive(Debug)]
pub enum PotatoGameError {
    ConcurrencyError,
}

impl Display for PotatoGameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PotatoGameError {}
