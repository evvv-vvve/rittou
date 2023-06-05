use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{CommandDataOption, CommandDataOptionValue};
use serenity::model::user::User;


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("scramblr")
        .description("Scramble up your messages and make a new one!")
        .create_option(|option|
            option
                .name("User")
                .description("The user to scramble your messages with")
                .kind(CommandOptionType::User)
                .required(false)
        )
}

pub async fn run(msg_author: &User, options: &[CommandDataOption]) -> String {
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

    match mash_messages(&msg_author, &provided_user) {
        Ok(content) => content,
        Err(scramblr_error) => format!("{:?}", scramblr_error)
    }
}

pub fn mash_messages(first_user: &User, second_user: &User) -> Result<String, ScramblrError> {
    if first_user.bot || second_user.bot {
        return Err(ScramblrError::IsBot);
    }

    Ok("pee".to_string())
}


#[derive(thiserror::Error, Debug)]
pub enum ScramblrError {
    #[error("One or more users is a bot")]
    IsBot,
    #[error("User {0} has too few messages")]
    TooFewMessages(User),
    #[error("No message matches were found")]
    NoMatches
}