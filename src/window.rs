use std::error::Error;
use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::Surface;

use winit::dpi::PhysicalSize;
use winit::keyboard::ModifiersState;
use winit::raw_window_handle::DisplayHandle;
use winit::window::Window;

use crate::app::Application;
use crate::render::Drawable;

pub struct WindowState {
    // Render Surface.
    surface: Surface<DisplayHandle<'static>, Arc<Window>>,

    /// winit Window.
    pub(crate) window: Arc<Window>,

    /// Window modifiers.
    pub(crate) modifiers: ModifiersState,
}

impl WindowState {
    pub fn new(app: &Application, window: Window) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(window);
        let surface = Surface::new(app.context.as_ref().unwrap(), Arc::clone(&window))?;

        //let size = window.inner_size();
        //background.resize(size);

        let state = WindowState {
            surface,
            window,
            modifiers: Default::default(),
        };

        Ok(state)
    }

    /// Resize the window to the new size.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let (width, height) = match (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
            (Some(width), Some(height)) => (width, height),
            _ => return,
        };

        self.surface
            .resize(width, height)
            .expect("failed to resize surface");

        self.window.request_redraw();
    }

    pub fn draw(&mut self, drawables: Vec<&mut dyn Drawable>) -> Result<(), Box<dyn Error>> {
        let mut buffer = self.surface.buffer_mut()?;

        for drawable in drawables {
            drawable.draw(&self.window, &mut buffer)?;
        }

        self.window.pre_present_notify();
        buffer.present()?;
        Ok(())
    }
}
