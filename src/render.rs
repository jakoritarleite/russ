use std::error::Error;
use std::sync::Arc;

use softbuffer::Buffer;
use winit::raw_window_handle::DisplayHandle;
use winit::window::Window;

pub trait Drawable {
    fn draw(
        &mut self,
        window: &Window,
        buffer: &mut Buffer<DisplayHandle<'static>, Arc<Window>>,
    ) -> Result<(), Box<dyn Error>>;
}
