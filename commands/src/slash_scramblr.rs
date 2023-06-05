use bot_data::scramblr::{ScramblrError, get_scrambled_message};
use bot_data::user_message_cache::{UserMessageData, UserMessageCache};
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::user::User;


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("scramblr")
        .description("Scramble up your messages and make a new one!")
        .create_option(|option| {
            option
                .name("user")
                .description("The user to scramble your messages with")
                .kind(CommandOptionType::User)
                .required(false)
        })
}

pub async fn run(msg_author: &User, user_message_cache: &UserMessageCache, options: &[CommandDataOption]) -> String {
    // get user-provided user, or default to message author
    let provided_user = match options.get(0) {
        Some(cmd_option) => {
            if let Some(cmd_option) = &cmd_option.resolved {
                if let CommandDataOptionValue::User(mentioned_user, _) = cmd_option {
                    mentioned_user.clone()
                } else {
                    msg_author.clone()
                }
            } else {
                msg_author.clone()
            }
        },
        None => msg_author.clone()
    };

    match message_masher(&msg_author, &provided_user, user_message_cache) {
        Ok(content) => content,
        Err(scramblr_error) => format!("{:?}", scramblr_error)
    }
}

pub fn message_masher(
    first_user: &User,
    second_user: &User,
    user_message_cache: &UserMessageCache
) -> Result<String, ScramblrError> {
    if first_user.bot || second_user.bot {
        return Err(ScramblrError::IsBot);
    }

    /*let user_a_count = if let Some(msgs) = user_message_cache.get_user_messages(first_user.id.0) {
        msgs.len()
    } else {
        0
    };

    let user_b_count = if let Some(msgs) = user_message_cache.get_user_messages(second_user.id.0) {
        msgs.len()
    } else {
        0
    };

    Ok(format!("{} ({} messages cached) and {} ({} messages cached)", 
        first_user.name, user_a_count,
        second_user.name, user_b_count
    ))*/

    get_scrambled_message(first_user, second_user, user_message_cache)
}