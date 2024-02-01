use std::env;

use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        id::{ChannelId, UserId},
    },
    prelude::*,
};

use serenity::utils::MessageBuilder;

// const OWNER_ID: UserId = UserId(323262113873264660); // ICoE
// const KICK_CHANNEL: ChannelId = ChannelId(889360426998046780); // ok.testing
// const LOG_CHANNEL: ChannelId = ChannelId(923761835583352973); // ok.testing

// const OWNER_ID: UserId = UserId(134509976956829697); // @typecasto#0517
// const KICK_CHANNEL: ChannelId = ChannelId(888501834312986635); // yuzu piracy
// const LOG_CHANNEL: ChannelId = ChannelId(923753624427982898);

async fn generate_kick_private_message(message: &Message, ctx: &Context) -> String {
    let name = if let Some(guild) = message.guild_id {
        guild.to_partial_guild(&ctx.http).await.ok().map(|x| x.name)
    } else {None}
        .unwrap_or("an unknown guild".to_string());
        // .unwrap(); // Either there is a string here or I've done something terribly wrong
    MessageBuilder::new()
        .push_line(format!(
            "You've been kicked from {} for being a suspected spambot.",
            name
        ))
        .push_line("Feel free to rejoin once you've secured your account.")
        // .push_line("You may want to check your recent DMs, to see if your account has sent any sketchy links.")
        .push_line(
            "Change your password, enable 2FA, and don't click on any sketchy links from now on.",
        )
        .build()
}

fn generate_kick_log_message(message: &Message, could_pm: bool) -> String {
    /*
    --- Spambot Kicked ---
    Username: `typecasto`
    ID: `10203040506`
    Sent a PM: No
    Avatar: https://example.com/avatar.png
     */
    MessageBuilder::new()
        .push_line("--- Spambot Kicked ---")
        .push("Username: ")
        .push_mono_line(&message.author.tag())
        .push("ID: ")
        .push_mono_line(&message.author.id)
        .push_line(format!(
            "Sent a PM: {}",
            if could_pm { "Yes" } else { "No" }
        ))
        .push_line(format!("Avatar: {}", &message.author.face()))
        .build()
}

struct Handler {
    kick_channel: ChannelId,
    log_channel: Option<ChannelId>,
    owner_id: UserId,
    // guild_id: GuildId,
    // exempt_roles: Vec<RoleId>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        // Stage 1: Detect
        // Duck out early if this isn't something we should handle
        if new_message.channel_id != self.kick_channel
            || new_message.author.id == self.owner_id
            || new_message.author.bot
        {
            // FEATURE: check for list of roles, rather than hardcoded ID
            return;
        }
        println!("Got message from: {}", new_message.author.tag());

        // Stage 2: Warn
        // Send them a private message
        // unfortunately we can't do this after we're sure we can ban them, since after we do that
        // we no longer share a server with them.
        let could_private_message;
        let private_message_text = generate_kick_private_message(&new_message, &ctx).await;
        if let Ok(dm_channel) = new_message.author.create_dm_channel(&ctx.http).await {
            could_private_message = dm_channel
                .send_message(&ctx.http, |m| m.content(private_message_text))
                .await
                .is_ok();
        } else {
            could_private_message = false;
        }

        // Stage 3: Ban
        // Ban them, deleting 1 day of messages and kicking them, then unban them.
        if let Some(guild) = &new_message.guild_id {
            if let Err(e) = guild
                .ban_with_reason(&ctx.http, &new_message.author.id, 1, "Spambot (autobanned)")
                .await
            {
                eprintln!("Failed to ban user, perms issue?");
                eprintln!("{:?}", e);
                return; // can't ban this user, return.
            }
            // todo? add a failsafe to DM the owner in case it fails to unban someone for some reason
            let _ = guild.unban(&ctx.http, &new_message.author.id).await;
        } else {
            eprintln!("Failed to find guild.")
        }

        // Stage 4: Log
        // Send a log message, simple.
        if let Some(log_channel) = self.log_channel {
            let Some(log_channel) = &ctx.cache.guild_channel(log_channel).await else {
                return;
            };
            log_channel
                .say(
                    &ctx.http,
                    generate_kick_log_message(&new_message, could_private_message),
                )
                .await
                .expect("Failed to send log message.");
        }
    }

    // Fired when bot is ready
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected and ready to go.", ready.user.name)
    }
}

#[tokio::main]
async fn main() {
    println!(
        "Starting {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );
    let _ = dotenvy::dotenv();

    // Config from environment
    // obvious hack is obvious
    let discord_token = env::var("DISCORD_TOKEN")
        .expect("Expected a discord token from environment variable $DISCORD_TOKEN.");
    // let bot_id = env::var("BOT_ID")
    //     .expect("Expected a discord token from environment variable $BOT_ID.")
    //     .parse::<u64>()
    //     .expect("Couldn't parse BOT_ID correctly.")
    //     .into();
    let owner_id = env::var("OWNER_ID")
        .expect("Expected a discord token from environment variable $OWNER_ID.")
        .parse::<u64>()
        .expect("Couldn't parse OWNER_ID correctly.")
        .into();
    let kick_channel = env::var("KICK_CHANNEL")
        .expect("Expected a discord token from environment variable $KICK_CHANNEL.")
        .parse::<u64>()
        .expect("Couldn't parse KICK_CHANNEL correctly.")
        .into();
    let log_channel = env::var("LOG_CHANNEL")
        .ok()
        .map(|x| x.parse::<u64>().expect("Couldn't parse LOG_CHANNEL correctly."))
        .map(Into::into);

    // Make bot
    let mut client = Client::builder(&discord_token)
        .event_handler(Handler {
            kick_channel,
            log_channel,
            owner_id
        }).await.expect("Couldn't build bot.");
        // .application_id(bot_id)
        
        // .await
        // .expect("bot create error.");

    // Start bot
    if let Err(error) = client.start().await {
        eprintln!("Client error: {:?}", error);
    }
}
