use std::{sync::Arc, collections::HashSet};

use discord_event_handler::DiscordEventHandler;
use serenity::{prelude::*, gateway::ShardManager, framework::StandardFramework, http::Http};

use bot_data::{config::{Config, ConfigData}};
use commands::{
    utility::*,
    fun::*
};

use bot_data::message_id_cache::{MessageIdCache, UserMessageData};

pub mod discord_event_handler;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[tokio::main]
async fn main() {
    let config = Config::from_file("config.toml");

    match config {
        Ok(config) => {
            let http = Http::new(config.get_token());

            // fetch owners and id
            let (owners, bot_id) = match http.get_current_application_info().await {
                Ok(info) => {
                    let mut owners = HashSet::new();

                    if let Some(team) = info.team {
                        owners.insert(team.owner_user_id);
                    } else if let Some(owner) = &info.owner {
                        owners.insert(owner.id);
                    }

                    match http.get_current_user().await {
                        Ok(bot_id) => (owners, bot_id.id),
                        Err(bot_id_err) => panic!("Could not access bot id: {:?}", bot_id_err)
                    }
                },
                Err(app_info_err) => panic!("Could not access application info: {:?}", app_info_err)
            };

            let framework = StandardFramework::new()
                .group(&UTILITY_GROUP)
                .group(&FUN_GROUP);

            framework.configure(|c| {
                c.with_whitespace(false)
                 .on_mention(Some(bot_id))
                 .prefixes(config.get_prefixes())
                 .owners(owners)
            });

            let intents = GatewayIntents::non_privileged()
                | GatewayIntents::MESSAGE_CONTENT
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::GUILD_MESSAGES;

            let mut client = Client::builder(config.get_token(), intents)
                .event_handler(DiscordEventHandler)
                .framework(framework)
                .await
                .expect("Couldn't create client!");
            
            client.cache.set_max_messages(256);

            // DATA INSERTION
            {
                let mut data = client.data.write().await;

                data.insert::<UserMessageData>(Arc::new(RwLock::new(MessageIdCache::new())));
                data.insert::<ConfigData>(Arc::new(config));
                data.insert::<ShardManagerContainer>(client.shard_manager.clone());
            }

            let shard_manager = client.shard_manager.clone();

            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.expect("Could not register ctrl+c handler");
                
                println!("Exit request (ctrl-c) received; safely shutting down");
                shard_manager.lock().await.shutdown_all().await;
            });

            if let Err(err) = client.start().await {
                println!("Error while running client: {err:?}")
            }
        },
        Err(config_error) => {
            println!("An error occurred while loading config: {config_error:#}")
        }
    }
}
