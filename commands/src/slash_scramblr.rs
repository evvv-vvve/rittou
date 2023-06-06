use bot_data::scramblr::get_scrambled_message;
use bot_data::user_message_cache::UserMessageCache;
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

    match get_scrambled_message(msg_author, &provided_user, user_message_cache) {
        Ok(content) => content,
        Err(scramblr_error) => format!("{:?}", scramblr_error)
    }
}