# temporary-anti-spam-bot
Stop discord spam with a honeypot channel

# Instructions
1. Make a Discord bot and add it to your server. It requires the following permissions (which sum to 3076):
   - Ban Members
   - Read Messages
   - Send Messages
2. Make a public channel called `#post-here-to-get-banned` (or similar) where all members can see and post, and post a warning there saying that it kicks anyone who posts here. 
3. Copy `example-.env` to `.env` and modify the config options.
    - You may also just set the relevant environment variables yourself, if you prefer.
4. Run `cargo run --release`. A Dockerfile and docker-compose.yml are also provided.
