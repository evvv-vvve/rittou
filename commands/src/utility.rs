use bot_data::user_message_cache::UserMessageData;
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
#[commands(ping, save, load)]
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

#[command]
#[owners_only]
/// The classic ping-pong
pub async fn save(ctx: &Context, msg: &Message) -> CommandResult {
    let msgs_lock = {
        let data_read = ctx.data.read().await;

        data_read.get::<UserMessageData>().expect("Expected UserMessageData").clone()
    };

    let _ = match msgs_lock.read().await.save_cache() {
        Ok(_) => msg.reply(&ctx.http, "Saved!").await,
        Err(e) => {
            let aaa = format!("Error while saving: {e:?}");
            msg.reply(&ctx.http, &aaa).await
        }
    };

    Ok(())
}

#[command]
#[owners_only]
/// The classic ping-pong
pub async fn load(ctx: &Context, msg: &Message) -> CommandResult {
    let msgs_lock = {
        let data_read = ctx.data.write().await;

        data_read.get::<UserMessageData>().expect("Expected UserMessageData").clone()
    };

    let _ = match msgs_lock.write().await.load_cache() {
        Ok(_) => msg.reply(&ctx.http, "Loaded!").await,
        Err(e) => {
            let aaa = format!("Error while loading: {e:?}");
            msg.reply(&ctx.http, &aaa).await
        }
    };

    Ok(())
}