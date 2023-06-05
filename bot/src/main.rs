use std::{sync::{Arc, mpsc::channel}, thread};

use data::user_message_cache::{UserMessageCache, UserMessageData};
use discord_event_handler::DiscordEventHandler;
use serenity::{framework::StandardFramework, client::bridge::gateway::ShardManager};
use serenity::prelude::*;

use config::{Config, ConfigData};
use commands::{
    utility::*,
    fun::*
};
use tokio::runtime::Runtime;


pub mod config;
pub mod discord_event_handler;
pub mod data;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[tokio::main]
async fn main() {
    let config = Config::from_file("config.toml");

    match config {
        Ok(config) => {
            let framework = StandardFramework::new()
                .configure(|conf| conf.prefixes(config.get_prefixes()))
                .group(&UTILITY_GROUP)
                .group(&FUN_GROUP);

            let intents = GatewayIntents::non_privileged()
                | GatewayIntents::MESSAGE_CONTENT
                | GatewayIntents::DIRECT_MESSAGES
                | GatewayIntents::GUILD_MESSAGES;

            let mut client = Client::builder(config.get_token(), intents)
                .event_handler(DiscordEventHandler)
                .framework(framework)
                .await
                .expect("Couldn't create client!");
            
            client.cache_and_http.cache.set_max_messages(256);

            // DATA INSERTION
            {
                let mut data = client.data.write().await;

                data.insert::<UserMessageData>(Arc::new(RwLock::new(UserMessageCache::new())));
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
