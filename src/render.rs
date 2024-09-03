use thiserror::Error;
use tiny_skia::Pixmap;
use winit::window::Window;

pub trait Drawable {
    fn draw(&mut self, window: &Window, buffer: &mut Pixmap) -> Result<(), DrawError>;
}

#[derive(Debug, Error)]
#[error("draw error")]
pub enum DrawError {
    #[error("An error ocurred when rendering the frame: {0}")]
    Render(#[from] pixels::Error),
}
