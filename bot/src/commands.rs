use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, Args};

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("Failed to parse {0} as a component")]
    ParseComponentError(String),
}

#[group]
#[commands(ping, emojify)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let start = std::time::Instant::now();
    let mut pong_msg = msg.reply(ctx, "Waiting...").await?;
    let elapsed = start.elapsed();

    pong_msg.edit(&ctx, |msg| {
        msg.content(format!("Pong! Took {}ms to respond", elapsed.as_millis()))
    }).await?;

    Ok(())
}

#[command]
/// Convert message text into emojis
async fn emojify(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let numbers = vec![ "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine" ];

    let msg_content = args.rest().to_lowercase();
    let mut new_msg_text = String::new();

    for chara in msg_content.chars().into_iter() {
        if !chara.is_ascii() {
            // if this character isn't ascii, attempt to just push it into the new msg
            match std::str::from_utf8(&[chara as u8]) {
                Ok(char_str) => new_msg_text.push_str(char_str),
                Err(_) => new_msg_text.push_str("?")
            }
        } else {
            if chara >= 'a' && chara <= 'z' {
                new_msg_text.push_str(format!(":regional_indicator_{chara}:").as_str())
            }
            else if chara >= '0' && chara <= '9' {
                new_msg_text.push_str(format!(":{}:", numbers[(chara as u32 - '0' as u32) as usize]).as_str())
            } else {
                match chara {
                    ' ' => new_msg_text.push_str("\t"),
                    '!' => new_msg_text.push_str(":exclamation:"),
                    '?' => new_msg_text.push_str(":question:"),
                     _  => {
                        match std::str::from_utf8(&[chara as u8]) {
                            Ok(char_str) => new_msg_text.push_str(char_str),
                            Err(_) => new_msg_text.push_str("?")
                        }
                     }
                }
            }
        }
    }

    msg.reply(&ctx, new_msg_text).await?;

    Ok(())
}