-- Add migration script here

CREATE TABLE players (
    discord_user_id VARCHAR(255) NOT NULL PRIMARY KEY,
    balance BIGINT NOT NULL,
    version BIGINT NOT NULL
)
