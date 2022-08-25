use std::fs;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};

pub mod config;

#[group]
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}

struct Handler;

#[async_trait]
impl EventHandler for Handler { }

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
      .configure(|conf| conf.prefix("y!"))
      .group(&GENERAL_GROUP);
    
    let config_contents = fs::read_to_string("config.toml")
       .expect("Could not read config file");

    let config: config::Config = toml::from_str(config_contents.as_str()).expect("Could not parse config file");
    
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(config.get_token(), intents)
      .event_handler(Handler)
      .framework(framework)
      .await
      .expect("Couldn't create client!");

    if let Err(err) = client.start().await {
      println!("Error while running client: {err:?}")
    }
}
