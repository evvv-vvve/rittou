use std::{collections::HashMap, sync::Arc};

use serde::{Serialize, Deserialize};
use serenity::{model::prelude::Message, prelude::{TypeMapKey, RwLock}};
use url::Url;

use crate::{encryption, config::Config};

#[derive(Clone, Serialize, Deserialize)]
pub struct CacheMessage {
    pub id: String,
    pub channel_id: String,
    pub time: i64,
    pub data: Vec<u8>,
    pub nonce: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageCacheData {
    pub version: u16,

    // <user_id, map<channel_id, Vec<msg>>>
    pub data: HashMap<String, HashMap<String, Vec<CacheMessage>>>
}

impl MessageCacheData {
    pub fn new() -> Self {
        Self {
            version: 1,
            data: HashMap::new()
        }
    }
}

#[derive(Clone)]
pub struct UserMessageCache {
    pub max_msgs: usize,
    pub messages: MessageCacheData
}

pub struct UserMessageData;

impl TypeMapKey for UserMessageData {
    type Value = Arc<RwLock<UserMessageCache>>;
}

#[derive(thiserror::Error, Debug)]
pub enum MessageCacheError {
    #[error("Faled to load message cache!")]
    FailedToLoadCache,

    #[error("An error occurred while encrypting/decrypting: {0}")]
    CryptionError(String),

    #[error("An error occurred while converting message data to toml")]
    TomlConvertError,

    #[error("An error occurred while parsing toml")]
    TomlParseError,

    #[error("An error occurred while reading toml")]
    TomlReadError,

    #[error("An error occurred while creating path")]
    PathCreateError,

    #[error("An error occurred while writing file")]
    FileWriteError,
}

impl UserMessageCache {
    pub fn new() -> Self {
        Self {
            max_msgs: 200,
            messages: MessageCacheData::new()
        }
    }

    pub fn save_cache(&self) -> Result<(), MessageCacheError> {
        match toml::to_string(&self.messages) {
            Ok(data) => {
                match std::fs::create_dir_all("data") {
                    Ok(_) => {
                        if let Err(_e) = std::fs::write("data/messages.toml", data) {
                            Err(MessageCacheError::FileWriteError)
                        } else {
                            Ok(())
                        }
                    },
                    Err(e) => {
                        println!("{e:?}");
                        Err(MessageCacheError::PathCreateError)
                    }
                }
            },
            Err(_e) => {
                Err(MessageCacheError::TomlConvertError)
            }
        }
    }

    pub fn load_cache(&mut self) -> Result<(), MessageCacheError> {
        if let Ok(contents) = std::fs::read_to_string("data/messages.toml") {
            if let Ok(cache) = toml::from_str::<MessageCacheData>(contents.as_str()) {
                self.messages = cache;
                Ok(())
            } else {
                Err(MessageCacheError::TomlParseError)
            }
        } else {
            Err(MessageCacheError::TomlReadError)
        }
    }

    pub fn add_or_update_msg(&mut self, message: &Message, config: &Config) -> Result<(), MessageCacheError> {
        let mut msg_content = message.content.clone();

        // skip cache if the message is a dm or from a bot
        if message.is_private() || message.author.bot {
            return Ok(());
        }

        // check if msg has a command prefix

        let mut split_msg = message.content.split(" ").collect::<Vec<&str>>();

        // remove any URLs from the message
        if let Some(mut indexes) = string_has_url(&msg_content) {
            indexes.sort();
            indexes.reverse();

            for index in indexes {
                split_msg.remove(index);
            }

            msg_content = split_msg.join(" ");
        }

        // skip if the message is too short
        if split_msg.len() < 3 {
            return Ok(());   
        }

        // check if a channel should be cached

        // check if a user has message caching enabled

        let (enc_data, nonce) = match encryption::encrypt(&message.content, config) {
            Ok(res) => {
                res
            },
            Err(e) => {
                return Err(MessageCacheError::CryptionError(e.to_string()))
            }
        };

        // find and modify an existing message,
        // or add a new one 
        self.messages.data
            .entry(message.author.id.get().to_string())
            .or_insert(HashMap::new())
            .entry(message.channel_id.get().to_string())
            .and_modify(|messages| {
                if let Some(msg) = messages.iter_mut().find(|msg|
                    msg.id == message.id.get().to_string()
                ) {
                    msg.data = enc_data.clone();
                    msg.time = message.timestamp.unix_timestamp();
                } else {
                    messages.push(CacheMessage {
                        id: message.id.get().to_string(),
                        channel_id: message.channel_id.get().to_string(),
                        time: message.timestamp.unix_timestamp(),
                        data: enc_data.clone(),
                        nonce: nonce.clone()
                    })
                }
            })
            .or_insert(vec![
                CacheMessage {
                    id: message.id.get().to_string(),
                    channel_id: message.channel_id.get().to_string(),
                    time: message.timestamp.unix_timestamp(),
                    data: enc_data.clone(),
                    nonce: nonce.clone()
                }
            ]);
        

        if let Some(mut user_messages) = self.get_user_messages_mut(message.author.id.get()) {
            user_messages.sort_by(|msg_a, msg_b| msg_a.time.cmp(&msg_b.time));

            if user_messages.len() > self.max_msgs {
                let amt_msgs_to_remove = user_messages.len() - self.max_msgs;

                for index in 0..amt_msgs_to_remove {
                    self.messages.data
                        .get_mut(&message.author.id.get().to_string())
                        .unwrap()
                        .get_mut(&message.channel_id.get().to_string())
                        .unwrap()
                        .remove(index);
                }
            }
        }

        Ok(())
    }

    pub fn get_user_messages(&self, user_id: u64) -> Option<Vec<&CacheMessage>> {
        if let Some(msgs) = self.messages.data.get(&user_id.to_string()) {
            let mut user_messages = Vec::new();

            for (_channel_id, messages) in msgs {
                user_messages.extend(messages.iter());
            }

            Some(user_messages)
        } else {
            None
        }
    }

    pub fn get_user_messages_mut(&self, user_id: u64) -> Option<Vec<&CacheMessage>> {
        if self.messages.data.contains_key(&user_id.to_string()) {
            let mut user_messages = Vec::new();

            for messages_map in self.messages.data.values() {
                for messages in messages_map.values() {
                    user_messages.extend(messages.iter());
                }
            }

            Some(user_messages.clone())
        } else {
            None
        }
    }

    pub fn remove_message(&mut self, message: &Message) {
        self.remove_message_by_id(message.author.id.get(), message.channel_id.get(), message.id.get());
    }

    pub fn remove_message_by_id(&mut self, user_id: u64, channel_id: u64, message_id: u64) {
        if let Some(msgs) = self.messages.data.get_mut(&user_id.to_string()) {
            if let Some(values) = msgs.get_mut(&channel_id.to_string()) {
                values.retain(|msg| msg.id != message_id.to_string());
            }
        }
    }

    pub fn remove_messages_in_channel(&mut self, channel_id: u64) {
        for (_, user_messages) in &mut self.messages.data {
            user_messages.remove(&channel_id.to_string());
        }
    }
}

/// Takes in string `content`, and checks for
/// urls.<br>Returns `None` if it doesn't, and
/// a vector of indexes if it does.
fn string_has_url(content: &str) -> Option<Vec<usize>> {
    let mut indexes = Vec::new();

    let split = content.split(" ");

    for (index, substr) in split.enumerate() {
        if let Ok(_) = Url::parse(substr) {
            indexes.push(index);
        }
    }

    if indexes.len() > 0 {
        Some(indexes)
    } else {
        None
    }
}