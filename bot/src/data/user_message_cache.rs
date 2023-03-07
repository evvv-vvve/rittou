use std::collections::HashMap;

use serenity::model::prelude::Message;
use url::Url;

pub struct CachedMessage {
    pub id: u64,
    pub time: i64,
    pub channel_id: u64,
    pub content: String,
}

pub struct MessageCacheData {
    pub version: u16,
    pub data: HashMap<u64, Vec<CachedMessage>>
}

impl MessageCacheData {
    pub fn new() -> Self {
        Self {
            version: 1,
            data: HashMap::new()
        }
    }
}

pub struct UserMessageCache {
    pub max_msgs: usize,
    pub messages: MessageCacheData
}

impl UserMessageCache {
    pub fn new() -> Self {
        Self {
            max_msgs: 5,
            messages: MessageCacheData::new()
        }
    }

    pub fn add_or_update_msg(&mut self, message: &Message) {
        let mut msg_content = message.content.clone();

        if message.is_private() || message.author.bot {
            return;
        }

        // check if msg has a command prefix

        if let Some(indexes) = string_has_url(&msg_content) {
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
        self.messages.data.entry(message.author.id.0).and_modify(|msgs| {
            if let Some(msg) = msgs.iter_mut().find(|msg| msg.id == message.id.0) {
                msg.content = msg_content.clone();
                msg.time = message.timestamp.unix_timestamp();
            } else {
                msgs.push(CachedMessage {
                    id: message.id.0,
                    time: message.timestamp.unix_timestamp(),
                    channel_id: message.channel_id.0,
                    content: msg_content.clone()
                })
            }
        })
        .or_insert(vec![
            CachedMessage {
                id: message.id.0,
                time: message.timestamp.unix_timestamp(),
                channel_id: message.channel_id.0,
                content: msg_content.clone()
            }
        ]);

        if let Some(msgs) = self.messages.data.get_mut(&message.author.id.0) {
            if msgs.len() > self.max_msgs {
                let amt_msgs_to_remove = msgs.len() - self.max_msgs;

                msgs.drain(0..=amt_msgs_to_remove);
            }
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