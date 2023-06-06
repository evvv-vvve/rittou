use serenity::all::Interaction;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponseMessage, CreateInteractionResponse};
use serenity::model::gateway::Ready;
use serenity::model::prelude::{GuildId, Message, MessageUpdateEvent};
use serenity::prelude::*;

use bot_data::user_message_cache::UserMessageData;

use crate::config::ConfigData;

pub struct DiscordEventHandler;

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let response = match command.data.name.as_str() {
                "cat" => Some(commands::slash_cat::run(&command.data.options()).await),
                "dog" => Some(commands::slash_dog::run(&command.data.options()).await),
                "scramblr" => {
                    // get user message cache
                    let user_message_cache = {
                        let data_read = ctx.data.read().await;

                        data_read.get::<UserMessageData>().expect("Expected UserMessageData").clone()
                    }.read().await.clone();

                    Some(commands::slash_scramblr::run(&command.user, &user_message_cache, &command.data.options()).await)
                },
                _ => None,
            };

            if let Some(content) = response {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);

                if let Err(response_error) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {}", response_error);
                }
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let msgs_lock = {
            let data_read = ctx.data.read().await;

            data_read.get::<UserMessageData>().expect("Expected UserMessageData").clone()
        };

        {
            let mut cache = msgs_lock.write().await;

            cache.add_or_update_msg(&msg);
        }
    }

    async fn message_update(
        &self,
        ctx: Context,
        _old_if_available: Option<Message>,
        new_message: Option<Message>,
        _event: MessageUpdateEvent,
    ) {
        cache_user_message(&ctx, &new_message).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let bot_config = {
            let data_read = ctx.data.read().await;

            data_read.get::<ConfigData>().expect("Expected ConfigData").clone()
        };

        if let Some(id) = bot_config.get_dev_guild_id() {
            let guild_id = GuildId::new(*id);

            let commands = guild_id.set_commands(
                &ctx.http,
                vec![
                    commands::slash_cat::register(),
                    commands::slash_dog::register(),
                    commands::slash_scramblr::register()
                ]
            )
            .await;

            println!("registered guild commands: {commands:#?}");
        } else {
            println!("dev guild id missing or invalid, skipping command registration");
        }

        /*let _guild_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::slash_cat::register(command);
            commands::slash_dog::register(command)
        })
        .await;
        
        if let Ok(cmds) = ctx.http.get_global_application_commands().await {
            for cmd in cmds {
                if let Ok(_) = Command::delete_global_application_command(&ctx.http, cmd.id).await {
                    println!("Removed global slash command w/id {}", cmd.id);
                } else {
                    println!("Could not remove global slash command w/id {}", cmd.id);
                }
            }
        }*/

    
        println!("{}: connected", ready.user.name);
    }
}

async fn cache_user_message(ctx: &Context, new_message: &Option<Message>) {
    let msgs_lock = {
        let data_read = ctx.data.read().await;

        data_read.get::<UserMessageData>().expect("Expected UserMessageData").clone()
    };

    println!("MSG UPDATE");
    if let Some(msg) = new_message {
        {
            let mut cache = msgs_lock.write().await;

            cache.add_or_update_msg(&msg);
        }
    } else {
        println!("not available");
    }   
}