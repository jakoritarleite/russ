use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub background: Background,
    pub widgets: Vec<Widget>,
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigError> {
        Ok(confy::load("russ", None)?)
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            background: Background::Color((0, 0, 0)),
            widgets: vec![Widget::Clock {
                position: Position::Center,
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Background {
    Image(String),
    Color((u8, u8, u8)),
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub enum Position {
    #[default]
    Center,
    XY {
        x: u32,
        y: u32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Widget {
    Clock { position: Position },
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ConfigError(#[from] confy::ConfyError);
