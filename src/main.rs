use std::env;

use serenity::model::id::{ChannelId, UserId};

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use serenity::utils::MessageBuilder;

// const OWNER_ID: UserId = UserId(323262113873264660); // ICoE
// const KICK_CHANNEL: ChannelId = ChannelId(889360426998046780); // ok.testing
// const LOG_CHANNEL: ChannelId = ChannelId(923761835583352973); // ok.testing

const OWNER_ID: UserId = UserId(134509976956829697); // @typecasto#0517
const KICK_CHANNEL: ChannelId = ChannelId(888501834312986635); // yuzu piracy
const LOG_CHANNEL: ChannelId = ChannelId(923753624427982898);

async fn generate_kick_private_message(message: &Message, ctx: &Context) -> String {
    let guild_name = &message
        .guild(&ctx.cache)
        .await
        .and_then(|g| Some(g.name))
        .or(Some(String::from("an unknown guild")))
        .unwrap(); // Either there is a string here or I've done something terribly wrong
    MessageBuilder::new()
        .push_line(format!(
            "You've been kicked from {} for being a suspected spambot.",
            guild_name
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
    Username: `User#1234`
    ID: `10203040506`
    Sent a PM: No
    Date: <t:1641780709:f>
     */
    MessageBuilder::new()
        .push_line("--- Spambot Kicked ---")
        .push("Username: ")
        // .push_mono_line(format!("{}#{:0>4}", &message.author.name, &message.author.discriminator))
        .push_mono_line(&message.author.tag())
        .push("ID: ")
        .push_mono_line(&message.author.id)
        .push_line(format!(
            "Sent a PM: {}",
            if could_pm { "Yes" } else { "No" }
        ))
        .push_line(format!("Avatar: {}", &message.author.face()))
        // .push_line(format!("Date: <t:{}:f>", &Utc::now().timestamp()))
        // .push_line("Original message:")
        // .push_safe(&message.content.replace("://", " : "))
        .build()
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, new_message: Message) {
        // Stage 1: Detect
        // Return if the message is in a different channel, or by a member of a protected role
        if new_message.channel_id != KICK_CHANNEL || new_message.author.id == OWNER_ID {
            // FEATURE: check for list of roles, rather than hardcoded ID
            return;
        }
        println!("Got message from: {}", new_message.author.tag());

        // Stage 2: Warn
        // Send them a private message
        // new_message.author.create_dm_channel(&ctx.http).await
        //     .and_then(|c| c.send_message(&ctx.http, |create_message| async move {
        //         create_message.content(generate_kick_private_message(&new_message, &ctx).await)
        //     }.await
        //     ));
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
        if let Some(guild) = &new_message.guild(&ctx.cache).await {
            if let Err(e) = guild
                .ban_with_reason(&ctx.http, &new_message.author.id, 1, "Spambot (autobanned)")
                .await
            {
                eprintln!("Failed to ban user, perms issue?");
                eprintln!("{:?}", e);
                return; // can't ban this user, return.
            }
            let _ = guild.unban(&ctx.http, &new_message.author.id).await;
        } else {
            eprintln!("Failed to find guild.")
        }

        // Stage 4: Log
        // Send a log message, simple.
        if let Some(log_channel) = &ctx.cache.guild_channel(LOG_CHANNEL).await {
            log_channel
                .say(
                    &ctx.http,
                    generate_kick_log_message(&new_message, could_private_message),
                )
                .await
                .expect("Failed to send log message.");
        } else {
            eprintln!("Failed to find log channel.")
        }
    }


    // Fired when bot is ready
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected and ready to go.", ready.user.name)

    }
}

#[tokio::main]
async fn main() {
    println!("Starting {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    dotenv::dotenv().ok();
    // Token from environment
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a discord token from environment variable $DISCORD_TOKEN.");

    // Make bot
    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(888489827492827206)
        .await
        .expect("bot create error.");

    // Start bot
    if let Err(error) = client.start().await {
        eprintln!("Client error: {:?}", error);
    }
}
