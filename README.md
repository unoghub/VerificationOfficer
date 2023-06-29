# Verification Officer

Discord bot for handling the verification of members that join the ÜNOG Discord server

## Configuration

Configuration is done using environment variables, `.env` files are supported

> These variables take effect after a restart

- `BOT_TOKEN`: The bot's token as shown in Discord Developer Portal
- `LOGGING_CHANNEL_ID`: ID of the channel where the errors will be logged
- `VERIFICATION_SUBMISSIONS_CHANNEL_ID`: ID of the channel where the verification submissions will be sent

### Required permissions

- In `LOGGING_CHANNEL_ID`:
    - Manage Webhooks
- In `VERIFICATION_SUBMISSIONS_CHANNEL_ID`:
    - View Channel
    - Send Messages

## Commands

Commands follow the format of `{path_to_exec} {command} {args}`, for example:
`./verification_officer create_verification_message 123`

These are one-off operations, start the executable without a command to run the bot

- `create_verification_message {channel_id}`: Sends the verification message in the given channel ID
    - Required permissions in `channel_id`:
        - View Channel
        - Send Messages
