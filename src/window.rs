use std::error::Error;
use std::num::NonZeroU32;
use std::sync::Arc;

use pixels::Pixels;
use pixels::SurfaceTexture;

use tiny_skia::Pixmap;
use winit::dpi::PhysicalSize;
use winit::keyboard::ModifiersState;
use winit::window::Window;

use crate::render::Drawable;

pub struct WindowState {
    frame_buffer: Pixels,
    drawing_buffer: Pixmap,

    /// winit Window.
    pub(crate) window: Arc<Window>,

    /// Window modifiers.
    pub(crate) modifiers: ModifiersState,
}

impl WindowState {
    pub fn new(window: Window) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(window);

        let surface_texture = SurfaceTexture::new(
            window.inner_size().width,
            window.inner_size().height,
            &window,
        );
        let frame_buffer = Pixels::new(
            window.inner_size().width,
            window.inner_size().height,
            surface_texture,
        )?;
        let drawing_buffer = Pixmap::new(window.inner_size().width, window.inner_size().height)
            .expect("creating drawing buffer");

        let state = WindowState {
            frame_buffer,
            drawing_buffer,
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

        self.frame_buffer
            .resize_surface(width.into(), height.into())
            .unwrap();
        self.frame_buffer
            .resize_buffer(width.into(), height.into())
            .unwrap();
        self.drawing_buffer = Pixmap::new(width.into(), height.into()).unwrap();

        self.window.request_redraw();
    }

    pub fn draw(&mut self, drawables: Vec<&mut dyn Drawable>) -> Result<(), Box<dyn Error>> {
        for drawable in drawables {
            drawable.draw(&mut self.drawing_buffer)?;
        }

        self.frame_buffer
            .frame_mut()
            .copy_from_slice(self.drawing_buffer.data());

        self.window.pre_present_notify();
        self.frame_buffer.render()?;
        Ok(())
    }
}
