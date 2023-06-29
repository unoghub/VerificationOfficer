# Verification Officer

Discord bot for handling the verification of members that join the ÜNOG Discord server

## Configuration

Configuration is done using environment variables, `.env` files are supported

> These variables take effect after a restart

- `BOT_TOKEN`: The bot's token as shown in Discord Developer Portal
- `GUILD_ID`: ID of the guild where the commands will be created
- `LOGGING_CHANNEL_ID`: ID of the channel where the errors will be logged

### Required permissions

- In `LOGGING_CHANNEL_ID`:
    - Manage Webhooks
