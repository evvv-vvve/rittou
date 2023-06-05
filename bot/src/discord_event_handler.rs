use serenity::async_trait;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::prelude::{GuildId, Message, MessageUpdateEvent};
use serenity::prelude::*;

use bot_data::user_message_cache::UserMessageData;

use crate::config::ConfigData;

pub struct DiscordEventHandler;

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let response = match command.data.name.as_str() {
                "cat" => Some(commands::slash_cat::run(&command.data.options).await),
                "dog" => Some(commands::slash_dog::run(&command.data.options).await),
                "scramblr" => {
                    // get user message cache
                    let user_message_cache = {
                        let data_read = ctx.data.read().await;

                        data_read.get::<UserMessageData>().expect("Expected UserMessageData").clone()
                    }.read().await.clone();

                    Some(commands::slash_scramblr::run(&command.user, &user_message_cache, &command.data.options).await)
                },
                _ => None,
            };

            if let Some(content) = response {
                let interaction_response = command.create_interaction_response(&ctx.http, |response| {
                    response.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(content))
                }).await;

                if let Err(response_error) = interaction_response {
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
            let guild_id = GuildId(*id);

            /*if let Ok(cmds) = ctx.http.get_guild_application_commands(*id).await {
                for cmd in cmds {
                    if let Ok(_) = guild_id.delete_application_command(&ctx.http, cmd.id).await {
                        println!("Removed global slash command w/id {}", cmd.id);
                    } else {
                        println!("Could not remove global slash command w/id {}", cmd.id);
                    }
                }
            }*/

            let _ = GuildId::set_application_commands(
                &guild_id,
                &ctx.http,
                |commands| {
                    commands.create_application_command(|command| commands::slash_cat::register(command));
                    commands.create_application_command(|command| commands::slash_dog::register(command));
                    commands.create_application_command(|command| commands::slash_scramblr::register(command))
                }
            )
            .await;

            if let Ok(cmds) = ctx.http.get_guild_application_commands(*id).await {
                for cmd in cmds {
                    println!("registered guild command {}", cmd.name);
                }
            }
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