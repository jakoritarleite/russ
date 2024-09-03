use serde::Deserialize;
use serde::Serialize;

use crate::widget::Position;

use super::Font;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "widget")]
pub enum Widget {
    Clock(ClockConfig),
    Text(TextConfig),
    Date(DateConfig),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClockConfig {
    pub show_seconds: bool,
    pub position: Position,
    #[serde(flatten)]
    pub font: Font,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextConfig {
    pub text: String,
    pub position: Position,
    #[serde(flatten)]
    pub font: Font,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateConfig {
    pub position: Position,
    #[serde(default = "default_date_format")]
    pub format: String,
    #[serde(flatten)]
    pub font: Font,
}

fn default_date_format() -> String {
    "%A - %B %d".to_string()
}
