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
        Ok(confy::load("russ", "config")?)
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            background: Background::Color((0, 0, 0)),
            widgets: vec![Widget::Clock {
                position: Position::XY { x: 50, y: 100 },
                font_size: 150.0,
                line_height: 1.0,
                show_seconds: false,
            }],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Background {
    Image(String),
    Color((u8, u8, u8)),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(tag = "type")]
pub enum Position {
    #[default]
    Center,
    XY {
        x: u32,
        y: u32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Widget {
    Clock {
        position: Position,
        font_size: f32,
        line_height: f32,
        show_seconds: bool,
    },
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ConfigError(#[from] confy::ConfyError);
