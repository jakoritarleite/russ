use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

pub mod clock;
pub mod date;
pub mod text;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(tag = "position")]
pub enum Position {
    #[default]
    Center,
    CenteredX {
        y: u32,
    },
    CenteredY {
        x: u32,
    },
    XY {
        x: u32,
        y: u32,
    },
}

#[derive(Debug, Error)]
#[error("widget error")]
pub struct WidgetError;
