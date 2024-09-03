use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct Font {
    pub line_height: f32,
    pub font_family: Option<String>,
    pub font_size: f32,
    pub font_weight: u16,
}
