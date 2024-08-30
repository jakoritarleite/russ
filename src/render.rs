use std::error::Error;
use tiny_skia::Pixmap;
use winit::window::Window;

pub trait Drawable {
    fn draw(&mut self, window: &Window, buffer: &mut Pixmap) -> Result<(), Box<dyn Error>>;
}
