use std::str::FromStr;
use std::time::Duration;

use serenity::async_trait;
use serenity::builder::{CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuOption};
use serenity::client::{Context, EventHandler};
use serenity::framework::StandardFramework;
use serenity::futures::StreamExt;
use serenity::model::application::component::ButtonStyle;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::channel::{Message, MessageFlags};
use serenity::prelude::*;

use config::Config;
use commands::*;

pub mod config;
pub mod commands;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
  async fn message(&self, ctx: Context, msg: Message) {
    if msg.content != "y!animal" {
      return;
    }

    // ask user fav animal
    let m = msg.channel_id
       .send_message(&ctx, |m| {
        m.content("Select your fav animal")
         .components(|c| c.add_action_row(Animal::action_row()))
       })
       .await
       .unwrap();
    
       // wait for user to select smth
       let mci =
       match m.await_component_interaction(&ctx).timeout(Duration::from_secs(60 * 3)).await {
           Some(ci) => ci,
           None => {
               m.reply(&ctx, "Timed out").await.unwrap();
               return;
           },
       };

       // data.custom_id contains the id of the component (here "animal_select")
        // and should be used to identify if a message has multiple components.
        // data.values contains the selected values from the menu
        let animal = Animal::from_str(mci.data.values.get(0).unwrap()).unwrap();

        // Acknowledge the interaction and edit the message
        mci.create_interaction_response(&ctx, |r| {
            r.kind(InteractionResponseType::UpdateMessage).interaction_response_data(|d| {
                d.content(format!("You chose: **{}**\nNow choose a sound!", animal))
                    .components(|c| c.add_action_row(Sound::action_row()))
            })
        })
        .await
        .unwrap();

        // Wait for multiple interactions

        let mut cib =
            m.await_component_interactions(&ctx).timeout(Duration::from_secs(60 * 3)).build();

        while let Some(mci) = cib.next().await {
            let sound = Sound::from_str(&mci.data.custom_id).unwrap();
            // Acknowledge the interaction and send a reply
            mci.create_interaction_response(&ctx, |r| {
                // This time we dont edit the message but reply to it
                r.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(
                    |d| {
                        // Make the message hidden for other users by setting `ephemeral(true)`.
                        d.ephemeral(true).content(format!("The **{}** says __{}__", animal, sound))
                    },
                )
            })
            .await
            .unwrap();
        }

        // Delete the orig message or there will be dangling components
        m.delete(&ctx).await.unwrap()
  }
}

#[tokio::main]
async fn main() {
    
  match Config::from_file("config.toml") {
    Ok(config) => {
      let framework = StandardFramework::new()
         .configure(|conf| conf.prefixes(config.get_prefixes()))
         .group(&GENERAL_GROUP);

      let intents = GatewayIntents::non_privileged()
         | GatewayIntents::MESSAGE_CONTENT
         | GatewayIntents::DIRECT_MESSAGES
         | GatewayIntents::GUILD_MESSAGES;

      let mut client = Client::builder(config.get_token(), intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Couldn't create client!");

      if let Err(err) = client.start().await {
        println!("Error while running client: {err:?}")
      }
    },
    Err(config_error) => {
      println!("An error occurred while loading config: {config_error:#}")
    }
  }
}
