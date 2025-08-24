# etbot

## Database

```sh
# Install sqlx CLI
cargo install sqlx-cli

# Create database
sqlx database create

# Drop database
sqlx database drop

# Apply migrations
sqlx migrate run

# Add new migration
sqlx migrate add <migration_name>
```
