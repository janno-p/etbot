-- Add migration script here

ALTER TABLE players
    ADD idle_since_ts BIGINT NOT NULL DEFAULT (unixepoch())
