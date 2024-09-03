use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

mod background;
mod font;
mod widget;

pub use background::Background;
pub use font::Font;
pub use widget::ClockConfig;
pub use widget::DateConfig;
pub use widget::TextConfig;
pub use widget::Widget;

use crate::widget::Position;

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
            widgets: vec![Widget::Clock(ClockConfig {
                show_seconds: false,
                position: Position::XY { x: 50, y: 100 },
                font: Font {
                    font_size: 150.0,
                    line_height: 1.0,
                    font_family: None,
                    font_weight: 100,
                },
            })],
        }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub struct ConfigError(#[from] confy::ConfyError);
