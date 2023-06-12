use std::{collections::HashMap, sync::Arc};

use serde::{Serialize, Deserialize};
use serenity::{model::prelude::Message, prelude::{TypeMapKey, RwLock}};
use url::Url;

use crate::{encryption, config::Config};

#[derive(Clone, Serialize, Deserialize)]
pub struct CacheMessage {
    pub id: u64,
    pub channel_id: u64,
    pub time: i64,
    pub data: Vec<u8>,
    pub nonce: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MessageCacheData {
    pub version: u16,

    // <user_id, map<channel_id, Vec<msg>>>
    pub data: HashMap<u64, HashMap<u64, Vec<CacheMessage>>>
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
}

impl UserMessageCache {
    pub fn new() -> Self {
        Self {
            max_msgs: 200,
            messages: MessageCacheData::new()
        }
    }

    pub fn load_cache(&self) -> Result<(), MessageCacheError> {
        if std::path::Path::new("data/messagecache.toml").exists() {
            
        }
        Ok(())
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
            .entry(message.author.id.get())
            .or_insert(HashMap::new())
            .entry(message.channel_id.get())
            .and_modify(|messages| {
                if let Some(msg) = messages.iter_mut().find(|msg|
                    msg.id == message.id.get()
                ) {
                    msg.data = enc_data.clone();
                    msg.time = message.timestamp.unix_timestamp();
                } else {
                    messages.push(CacheMessage {
                        id: message.id.get(),
                        channel_id: message.channel_id.get(),
                        time: message.timestamp.unix_timestamp(),
                        data: enc_data.clone(),
                        nonce: nonce.clone()
                    })
                }
            })
            .or_insert(vec![
                CacheMessage {
                    id: message.id.get(),
                    channel_id: message.channel_id.get(),
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
                        .get_mut(&message.author.id.get())
                        .unwrap()
                        .get_mut(&message.channel_id.get())
                        .unwrap()
                        .remove(index);
                }
            }
        }

        Ok(())
    }

    pub fn get_user_messages(&self, user_id: u64) -> Option<Vec<&CacheMessage>> {
        if let Some(msgs) = self.messages.data.get(&user_id) {
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
        if self.messages.data.contains_key(&user_id) {
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
        if let Some(msgs) = self.messages.data.get_mut(&user_id) {
            if let Some(values) = msgs.get_mut(&channel_id) {
                values.retain(|msg| msg.id != message_id);
            }
        }
    }

    pub fn remove_messages_in_channel(&mut self, channel_id: u64) {
        for (_, user_messages) in &mut self.messages.data {
            user_messages.remove(&channel_id);
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