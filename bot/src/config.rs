#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    token: String,
}

impl Config {
    pub fn new(token: &str) -> Self {
        Self {
            token: String::from(token)
        }
    }

    pub fn get_token(&self) -> String { self.token.clone() }
}