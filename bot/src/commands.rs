use std::str::FromStr;

use serenity::builder::{CreateSelectMenuOption, CreateSelectMenu, CreateActionRow, CreateButton};
use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{StandardFramework, CommandResult};
use serenity::model::interactions::message_component::SelectMenu;
use serenity::model::prelude::component::ButtonStyle;

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
#[commands(ping)]
struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;

    Ok(())
}