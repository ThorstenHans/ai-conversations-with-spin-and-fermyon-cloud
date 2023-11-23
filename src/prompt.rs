use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Prompt {
    pub question: String,
}
