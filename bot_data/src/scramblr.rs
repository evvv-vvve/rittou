use rand::{seq::SliceRandom, thread_rng, Rng};
use serenity::model::user::User;

use crate::user_message_cache::UserMessageCache;

#[derive(thiserror::Error, Debug)]
pub enum ScramblrError {
    #[error("One or more users is a bot")]
    IsBot,
    #[error("User {0} has too few messages. Minimum cached message count is `3`")]
    TooFewMessages(User),
    #[error("No message matches were found")]
    NoMatches
}

pub fn get_scrambled_message(
    user_a: &User,
    user_b: &User,
    user_message_cache: &UserMessageCache
) -> Result<String, ScramblrError> {
    if user_a.bot || user_b.bot {
        return Err(ScramblrError::IsBot);
    }

    let user_a_messages = user_message_cache.get_user_messages(user_a.id.0).unwrap_or(Vec::new());
    let user_b_messages = user_message_cache.get_user_messages(user_b.id.0).unwrap_or(Vec::new());

    if user_a_messages.len() <= 3 {
        return Err(ScramblrError::TooFewMessages(user_a.clone()))
    }

    if user_b_messages.len() <= 3 {
        return Err(ScramblrError::TooFewMessages(user_b.clone()))
    }

    let mut scramble_tries = 0;
    let mut rng = thread_rng();

    let mut last_msg = None;

    while scramble_tries < 25 {
        // choose a random message
        let user_a_msg = user_a_messages.choose(&mut rng);
        let user_b_msg = user_b_messages.choose(&mut rng);

        if user_a_msg.is_some() && user_b_msg.is_some() {
            let msg_a = user_a_msg.unwrap();
            let msg_b = user_b_msg.unwrap();

            if msg_a.id == msg_b.id {
                // skip iteration if message IDs match
                continue;
            }

            // split the messages
            let lower_a = msg_a.content.to_lowercase();
            let lower_b = msg_b.content.to_lowercase();

            let msg_a_split = lower_a.split(" ").collect::<Vec<&str>>();
            let msg_b_split = lower_b.split(" ").collect::<Vec<&str>>();

            let mut word_matches = Vec::new();
            
            // add any word matches to a list
            for split_a in msg_a_split {
                if msg_b_split.contains(&split_a) {
                    if !word_matches.contains(&split_a) {
                        word_matches.push(split_a);
                    }
                }
            }

            // if there are any word matches, create a scrambled message
            if word_matches.len() > 0 {
                let mut scrambled_msg = make_scrambled_message(&lower_a, &lower_b, word_matches.choose(&mut rng).unwrap());

                // truncate msg if it might be too long
                if scrambled_msg.len() > 2000 {
                    scrambled_msg = scrambled_msg.chars().take(1997).collect::<String>();
                    scrambled_msg.push_str("...");
                }

                // make sure message isnt just a repeat of either users msgs
                if scrambled_msg.to_lowercase() == msg_a.content.to_lowercase() ||
                   scrambled_msg.to_lowercase() == msg_b.content.to_lowercase() {
                    scramble_tries += 1;
                    
                    last_msg = Some(scrambled_msg);
                    continue;
                }

                return Ok(scrambled_msg);
            }

            scramble_tries += 1;
        }
    }

    if let Some(msg) = last_msg {
        Ok(msg)
    } else {
        Err(ScramblrError::NoMatches)
    }
}

fn make_scrambled_message(
    msg_a: &str,
    msg_b: &str,
    matched_word: &str
) -> String {
    println!("{}", matched_word);

    let indexes_a = get_word_indexes(msg_a, matched_word);
    let indexes_b = get_word_indexes(msg_b, matched_word);

    let mut rng = thread_rng();

    let split_a = split_at_word_index(msg_a, *indexes_a.choose(&mut rng).unwrap());
    let split_b = split_at_word_index(msg_b, *indexes_b.choose(&mut rng).unwrap());

    let first_part;
    let second_part;

    // decide the order to mash messages
    if rng.gen() {
        first_part = split_a.0.as_str();

        if !split_b.1.is_empty() {
            second_part = split_b.1.as_str();
        } else {
            second_part = split_b.0.as_str();
        }
    } else {
        first_part = split_b.0.as_str();

        if !split_a.1.is_empty() {
            second_part = split_a.1.as_str();
        } else {
            second_part = split_a.0.as_str();
        }
    }

    format!("{} {} {}", first_part, matched_word, second_part)
}

fn split_at_word_index(msg: &str, word_index: usize) -> (String, String) {
    let mut first_part = String::new();

    let split_msg = msg.split(" ").collect::<Vec<&str>>();

    let mut building_string = String::new();

    for (index, word) in split_msg.into_iter().enumerate() {
        if index == word_index {
            if building_string.len() > 0 {
                building_string.remove(building_string.len() - 1);
            }

            first_part = building_string.clone();
            building_string = String::new();
            continue;
        }

        building_string.push_str(&format!("{} ", word));
    }

    if building_string.len() > 0 {
        building_string.remove(building_string.len() - 1);
    }

    (first_part, building_string)
}

fn get_word_indexes(msg: &str, match_word: &str) -> Vec<usize> {
    let mut indexes = Vec::new();

    let lower_msg = msg.to_lowercase();
    let split_msg = lower_msg.split(" ").collect::<Vec<&str>>();

    for (index, word) in split_msg.into_iter().enumerate() {
        if word == match_word.to_lowercase() {
            indexes.push(index);
        }
    }

    indexes
}