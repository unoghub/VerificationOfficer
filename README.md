# Verification Officer

Discord bot for handling the verification of members that join the ÜNOG Discord server

## Configuration

Configuration is done using environment variables, `.env` files are supported

> These variables take effect after a restart

- `BOT_TOKEN`: The bot's token as shown in Discord Developer Portal
- `LOGGING_CHANNEL_ID`: ID of the channel where the errors will be logged
- `VERIFICATION_SUBMISSIONS_CHANNEL_ID`: ID of the channel where the verification submissions will be sent
- `VERIFICATION_APPROVALS_CHANNEL_ID`: ID of the channel where the users will be informed that the verification was approved

### Required permissions

- In `LOGGING_CHANNEL_ID`:
    - Manage Webhooks
- In `VERIFICATION_SUBMISSIONS_CHANNEL_ID` and `VERIFICATION_APPROVALS_CHANNEL_ID`:
    - View Channel
    - Send Messages

## Commands

Commands follow the format of `{path_to_exec} {command} {args}`, for example:
`./verification_officer create_verification_message 123`

These are one-off operations, start the executable without a command to run the bot

Permissions listed as required under the commands aren't required after the command has run

- `create_verification_message {channel_id}`: Sends the verification message in the given channel ID
    - Required permissions in `channel_id`:
        - View Channel
        - Send Messages

## Compilation

> On Linux, make sure you have `build-essential` installed

> On Windows, MSVC is required, Rust installer will direct you to this

1. [Install Rust](https://www.rust-lang.org/tools/install)
2. Clone this repository and `cd` into it
3. Run `cargo build --release`
4. The executable is at `target/release/verification_officer`
