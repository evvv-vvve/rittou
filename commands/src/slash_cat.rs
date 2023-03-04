use crate::fetch_error::FetchError;

use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;


#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CatObject {
    pub id: String,
    pub url: String,
    pub width: i32,
    pub height: i32,
}


pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("cat").description("Retrieve a random picture of a cat")
}

pub async fn run(_options: &[CommandDataOption]) -> String {
    match get_cat().await {
        Ok(cat) => cat.url,
        Err(e) => e.to_string()
    }
}

pub async fn get_cat() -> Result<CatObject, FetchError> {
    match reqwest::get("https://api.thecatapi.com/v1/images/search").await {
        Ok(response) => {
            match response.text().await {
                Ok(data) => {
                    match serde_json::from_str::<Vec<CatObject>>(data.as_str()) {
                        Ok(cat) => Ok(cat.first().unwrap().to_owned()),
                        Err(e) => Err(FetchError::ParseError(e))
                    }
                },
                Err(e) => Err(FetchError::DecodeError(e))
            }
        },
        Err(e) => Err(FetchError::RequestError(e))
    }
}