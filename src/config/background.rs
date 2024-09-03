use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub enum Background {
    Image(String),
    Color((u8, u8, u8)),
}
