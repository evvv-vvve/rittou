use crate::fetch_error::FetchError;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;


#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct DogObject {
    pub message: String,
    pub status: String,
}


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("dog").description("Retrieve a random picture of a dog")
}

pub async fn run(_options: &[CommandDataOption]) -> String {
    match get_dog().await {
        Ok(dog) => dog.message,
        Err(e) => e.to_string()
    }
}

pub async fn get_dog() -> Result<DogObject, FetchError> {
    match reqwest::get("https://dog.ceo/api/breeds/image/random").await {
        Ok(response) => {
            match response.text().await {
                Ok(data) => {
                    match serde_json::from_str::<DogObject>(data.as_str()) {
                        Ok(dog) => Ok(dog.to_owned()),
                        Err(e) => Err(FetchError::ParseError(e))
                    }
                },
                Err(e) => Err(FetchError::DecodeError(e))
            }
        },
        Err(e) => Err(FetchError::RequestError(e))
    }
}