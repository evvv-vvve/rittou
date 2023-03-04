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
#[commands(emojify)]
pub struct Fun;

#[command]
/// Convert message text into emojis
async fn emojify(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let text = args.rest().to_string();

    msg.reply(&ctx, convert_text_to_emojis(text)).await?;

    Ok(())
}

pub fn convert_text_to_emojis(text: String) -> String {
    let numbers = vec![ "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine" ];

    let mut new_text = String::new();

    for chara in text.to_lowercase().chars().into_iter() {
        // if this character isn't ascii, just push it into the new msg
        if !chara.is_ascii() {
            new_text.push(chara);
        } else {
            if chara >= 'a' && chara <= 'z' {
                new_text.push_str(format!(":regional_indicator_{chara}:").as_str())
            }
            else if chara >= '0' && chara <= '9' {
                new_text.push_str(format!(":{}:", numbers[(chara as u32 - '0' as u32) as usize]).as_str())
            } else {
                match chara {
                    ' ' => new_text.push_str("\t"),
                    '!' => new_text.push_str(":exclamation:"),
                    '?' => new_text.push_str(":question:"),
                     _  => {
                        match std::str::from_utf8(&[chara as u8]) {
                            Ok(char_str) => new_text.push_str(char_str),
                            Err(_) => new_text.push_str("?")
                        }
                     }
                }
            }
        }
    }

    new_text
}