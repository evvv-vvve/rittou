use std::time::{Duration, Instant};

use serenity::{
    client::Context,
    framework::standard::{
        CommandResult,
        Args,
        macros::{
            command,
            group
        }
    },
    model::channel::Message
};

#[group]
#[commands(ping)]
pub struct Utility;

#[command]
/// The classic ping-pong
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let start = std::time::Instant::now();
    let mut pong_msg = msg.reply(ctx, "Waiting...").await?;
    let elapsed = start.elapsed();

    pong_msg.edit(&ctx, |msg| {
        msg.content(format!("Pong! Took {}ms to respond", elapsed.as_millis()))
    }).await?;

    Ok(())
}