use serenity::async_trait;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::prelude::{GuildId, Message, MessageUpdateEvent};
use serenity::prelude::*;

use lazy_static::lazy_static;

use crate::config::{ConfigError, Config, self};
use crate::data::user_message_cache::UserMessageCache;

pub struct DiscordEventHandler;

lazy_static! {
    pub static ref USR_MSG_CACHE: RwLock<UserMessageCache> = RwLock::new(UserMessageCache::new());
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "cat" => commands::slash_cat::run(&command.data.options).await,
                "dog" => commands::slash_dog::run(&command.data.options).await,
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        USR_MSG_CACHE.write().await.add_or_update_msg(&msg);
    }

    async fn message_update(
        &self,
        ctx: Context,
        old_if_available: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        println!("MSG UPDATE");
        if let Some(msg) = new {
            
            USR_MSG_CACHE.write().await.add_or_update_msg(&msg);
        } else {
            println!("not available");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        if let Ok(config) = &*config::CONFIG {
            if let Some(id) = config.get_dev_guild_id() {
                let guild_id = GuildId(*id);

                let _ = GuildId::set_application_commands(
                    &guild_id,
                    &ctx.http,
                    |commands| {
                        commands.create_application_command(|command| commands::slash_cat::register(command));
                        commands.create_application_command(|command| commands::slash_dog::register(command))
                    }
                )
                .await;
            } else {
                println!("dev guild id missing or invalid, skipping command registration");
            }
        } else {
            println!("Config file missing or invalid, skipping command registration");
        }

        /*let _guild_command = Command::create_global_application_command(&ctx.http, |command| {
            commands::slash_cat::register(command);
            commands::slash_dog::register(command)
        })
        .await;*/
        
        /*if let Ok(cmds) = ctx.http.get_global_application_commands().await {
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