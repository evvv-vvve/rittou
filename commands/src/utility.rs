use serenity::{
    client::Context,
    framework::standard::{
        CommandResult,
        macros::{
            command,
            group
        }
    },
    model::channel::Message, builder::EditMessage
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

    let msg_content = format!("Pong! Took {}ms to respond", elapsed.as_millis());
    
    pong_msg.edit(&ctx, EditMessage::new().content(msg_content)).await?;

    Ok(())
}