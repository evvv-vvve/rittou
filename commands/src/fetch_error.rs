/// Errors that can occur while fetching
#[derive(thiserror::Error, Debug)]
pub enum FetchError {
    /// Returned when an http GET request fails
    #[error("Could not make request: {0}")]
    RequestError(reqwest::Error),

    /// Returned when decoding an http GET request
    /// fails
    #[error("Could not decode HTTP request: {0}")]
    DecodeError(reqwest::Error),
    

    /// Returned when cat cannot be converted
    /// to a CatObject
    #[error("Could not parse request to DogObject: {0}")]
    ParseError(serde_json::Error)
}