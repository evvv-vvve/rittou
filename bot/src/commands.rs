use std::str::FromStr;
use std::time::Duration;

use serenity::builder::{CreateSelectMenuOption, CreateSelectMenu, CreateActionRow, CreateButton};
use serenity::client::Context;
use serenity::futures::StreamExt;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult};
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::InteractionResponseType;

#[derive(thiserror::Error, Debug)]
pub enum CommandError {
    #[error("Failed to parse {0} as a component")]
    ParseComponentError(String),
}

#[derive(Debug)]
pub enum Animal {
    Cat,
    Dog,
    Horse,
    Goat,
}

impl std::fmt::Display for Animal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cat => write!(f, "cat"),
            Self::Dog => write!(f, "dog"),
            Self::Horse => write!(f, "horse"),
            Self::Goat => write!(f, "goat")
        }
    }
}

impl FromStr for Animal {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cat" => Ok(Self::Cat),
            "dog" => Ok(Self::Dog),
            "horse" => Ok(Self::Horse),
            "goat" => Ok(Self::Goat),
            _ => Err(Self::Err::ParseComponentError(s.to_string()))
        }
    }
}

impl Animal {
    fn emoji(&self) -> char {
        match self {
            Self::Cat => '🐈',
            Self::Dog => '🐕',
            Self::Horse => '🐎',
            Self::Goat => '🐐',
        }
    }

    fn menu_option(&self) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();

        // shown to the user
        opt.label(format!("{} {}", self.emoji(), self));

        // identify selected val
        opt.value(self.to_string().to_ascii_lowercase());
        opt
    }

    fn select_menu() -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("animal_select");
        menu.placeholder("No animal selected");
        menu.options(|f| {
            f.add_option(Self::Cat.menu_option())
             .add_option(Self::Dog.menu_option())
             .add_option(Self::Horse.menu_option())
             .add_option(Self::Goat.menu_option())
        });

        menu
    }

    pub fn action_row() -> CreateActionRow {
        let mut action_row = CreateActionRow::default();

        action_row.add_select_menu(Self::select_menu());

        action_row
    }
}

#[derive(Debug)]
pub enum Sound {
    Meow,
    Woof,
    Neigh,
    Baaa,
}

impl std::fmt::Display for Sound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Meow => write!(f, "meow"),
            Self::Woof => write!(f, "woof"),
            Self::Neigh => write!(f, "neigh"),
            Self::Baaa => write!(f, "baaa"),
        }
    }
}

impl Sound {
    fn emoji(&self) -> char {
        match self {
            Self::Meow => Animal::Cat.emoji(),
            Self::Woof => Animal::Dog.emoji(),
            Self::Neigh => Animal::Horse.emoji(),
            Self::Baaa => Animal::Goat.emoji(),
        }
    }

    fn button(&self) -> CreateButton {
        let mut butt = CreateButton::default();
        butt.custom_id(self.to_string().to_ascii_lowercase());
        butt.emoji(self.emoji());
        butt.label(self);
        butt.style(ButtonStyle::Primary);
        
        butt
    }

    pub fn action_row() -> CreateActionRow {
        let mut action_row = CreateActionRow::default();
        
        // up to 5 buttons per action row
        action_row.add_button(Sound::Meow.button());
        action_row.add_button(Sound::Woof.button());
        action_row.add_button(Sound::Neigh.button());
        action_row.add_button(Sound::Baaa.button());
        
        action_row
    }
}

impl FromStr for Sound {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "meow" => Ok(Sound::Meow),
            "woof" => Ok(Sound::Woof),
            "neigh" => Ok(Sound::Neigh),
            "baaaa" => Ok(Sound::Baaa),
            _ => Err(Self::Err::ParseComponentError(s.to_string())),
        }
    }
}

#[group]
#[commands(ping, animal)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    let start = std::time::Instant::now();
    let mut pong_msg = msg.reply(ctx, "Waiting...").await?;
    let elapsed = start.elapsed();

    pong_msg.edit(&ctx, |msg| {
        msg.content(format!("Pong! Took {}ms to respond", elapsed.as_millis()))
    }).await?;

    Ok(())
}

#[command]
async fn animal(ctx: &Context, msg: &Message) -> CommandResult {
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
               return Ok(());
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
        m.delete(&ctx).await.unwrap();

    Ok(())
}