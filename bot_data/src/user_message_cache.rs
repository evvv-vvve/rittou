use std::{collections::HashMap, sync::Arc};

use serenity::{model::prelude::Message, prelude::{TypeMapKey, RwLock}};
use url::Url;

#[derive(Clone)]
pub struct CachedMessage {
    pub id: u64,
    pub channel_id: u64,
    pub time: i64,
    pub content: String,
}

#[derive(Clone)]
pub struct MessageCacheData {
    pub version: u16,

    // <user_id, map<channel_id, Vec<msg>>>
    pub data: HashMap<u64, HashMap<u64, Vec<CachedMessage>>>
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

impl UserMessageCache {
    pub fn new() -> Self {
        Self {
            max_msgs: 200,
            messages: MessageCacheData::new()
        }
    }

    pub fn add_or_update_msg(&mut self, message: &Message) {
        let mut msg_content = message.content.clone();

        if message.is_private() || message.author.bot {
            return;
        }

        // check if msg has a command prefix
        if let Some(mut indexes) = string_has_url(&msg_content) {
            indexes.sort();
            indexes.reverse();

            let mut split_msg = msg_content.split(" ").collect::<Vec<&str>>();

            for index in indexes {
                split_msg.remove(index);
            }

            msg_content = split_msg.join(" ");
        }

        // check if a channel should be cached

        // check if a user has message caching enabled

        // find and modify an existing message,
        // or add a new one 
        self.messages.data
            .entry(message.author.id.0)
            .or_insert(HashMap::new())
            .entry(message.channel_id.0)
            .and_modify(|messages| {
                if let Some(msg) = messages.iter_mut().find(|msg|
                    msg.id == message.id.0
                ) {
                    msg.content = msg_content.clone();
                    msg.time = message.timestamp.unix_timestamp();
                } else {
                    messages.push(CachedMessage {
                        id: message.id.0,
                        channel_id: message.channel_id.0,
                        time: message.timestamp.unix_timestamp(),
                        content: msg_content.clone()
                    })
                }
            })
            .or_insert(vec![
                CachedMessage {
                    id: message.id.0,
                    channel_id: message.channel_id.0,
                    time: message.timestamp.unix_timestamp(),
                    content: msg_content.clone()
                }
            ]);
        

        if let Some(mut user_messages) = self.get_user_messages_mut(message.author.id.0) {
            user_messages.sort_by(|msg_a, msg_b| msg_a.time.cmp(&msg_b.time));

            if user_messages.len() > self.max_msgs {
                let amt_msgs_to_remove = user_messages.len() - self.max_msgs;

                for index in 0..amt_msgs_to_remove {
                    self.messages.data
                        .get_mut(&message.author.id.0)
                        .unwrap()
                        .get_mut(&message.channel_id.0)
                        .unwrap()
                        .remove(index);
                }
            }
        }
    }

    pub fn get_user_messages(&self, user_id: u64) -> Option<Vec<&CachedMessage>> {
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

    pub fn get_user_messages_mut(&self, user_id: u64) -> Option<Vec<&CachedMessage>> {
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
        self.remove_message_by_id(message.author.id.0, message.channel_id.0, message.id.0);
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