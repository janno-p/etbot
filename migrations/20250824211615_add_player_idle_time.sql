-- Add migration script here

CREATE TABLE new_players (
    discord_user_id VARCHAR(255) NOT NULL PRIMARY KEY,
    balance BIGINT NOT NULL,
    last_feed_ts BIGINT NOT NULL,
    version BIGINT NOT NULL,
	idle_since_ts BIGINT NOT NULL DEFAULT (unixepoch())
);

INSERT INTO new_players (discord_user_id, balance, last_feed_ts, version)
SELECT discord_user_id, balance, last_feed_ts, version FROM players;

DROP TABLE players;
ALTER TABLE new_players RENAME TO players;
