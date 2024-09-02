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
                show_seconds: false,
                font: FontConfig {
                    font_size: 150.0,
                    line_height: 1.0,
                    font_family: None,
                    font_weight: 100,
                },
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
        show_seconds: bool,
        #[serde(flatten)]
        font: FontConfig,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FontConfig {
    pub line_height: f32,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub font_weight: u16,
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ConfigError(#[from] confy::ConfyError);
