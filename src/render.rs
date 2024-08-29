use std::error::Error;
use tiny_skia::Pixmap;

pub trait Drawable {
    fn draw(&mut self, buffer: &mut Pixmap) -> Result<(), Box<dyn Error>>;
}
