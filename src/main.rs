use std::collections::HashMap;
use std::error::Error;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use chrono::Timelike;
use cosmic_text::Attrs;
use cosmic_text::Buffer;
use cosmic_text::Color;
use cosmic_text::FontSystem;
use cosmic_text::Metrics;
use cosmic_text::Shaping;
use cosmic_text::SwashCache;
use image::DynamicImage;
use image::GenericImageView;
use image::ImageReader;
use softbuffer::Context;
use softbuffer::Surface;
use tiny_skia::Paint;
use tiny_skia::PixmapMut;
use tiny_skia::Rect;
use tiny_skia::Transform;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::keyboard::Key;
use winit::keyboard::ModifiersState;
use winit::platform::startup_notify::EventLoopExtStartupNotify;
use winit::platform::startup_notify::WindowAttributesExtStartupNotify;
use winit::raw_window_handle::DisplayHandle;
use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Fullscreen;
use winit::window::Window;
use winit::window::WindowAttributes;
use winit::window::WindowId;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let mut app = Application::new(&event_loop);
    Ok(event_loop.run_app(&mut app)?)
}

fn get_time() -> String {
    let dt = chrono::Local::now();

    format!("{}:{}:{}", dt.hour(), dt.minute(), dt.second())
}

struct Application {
    windows: HashMap<WindowId, WindowState>,

    // Drawing context.
    context: Option<Context<DisplayHandle<'static>>>,

    font_system: Arc<Mutex<FontSystem>>,
    swash_cache: Arc<Mutex<SwashCache>>,
    widget_buffer: Arc<Mutex<Buffer>>,
}

impl Application {
    fn new(event_loop: &EventLoop<()>) -> Self {
        // SAFETY: the context is dropped right before the event loop is stopped, thus making it
        // safe.
        let context = Some(
            Context::new(unsafe {
                std::mem::transmute::<DisplayHandle<'_>, DisplayHandle<'static>>(
                    event_loop.display_handle().unwrap(),
                )
            })
            .unwrap(),
        );

        let mut font_system = FontSystem::new();
        let swash_cache = Arc::new(Mutex::new(SwashCache::new()));

        // TODO how this line height works???
        let mut widget_buffer = Buffer::new(&mut font_system, Metrics::new(150.0, 1.0));
        widget_buffer.set_text(
            &mut font_system,
            &get_time(),
            Attrs::new().family(cosmic_text::Family::Monospace),
            Shaping::Advanced,
        );

        let font_system = Arc::new(Mutex::new(font_system));
        let widget_buffer = Arc::new(Mutex::new(widget_buffer));

        {
            let event_loop_proxy = event_loop.create_proxy();
            let widget_buffer_clone = widget_buffer.clone();
            let font_system_clone = font_system.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));

                let mut font_system = font_system_clone.lock().unwrap();
                let mut buffer = widget_buffer_clone.lock().unwrap();
                buffer.set_text(
                    &mut font_system,
                    &get_time(),
                    Attrs::new().family(cosmic_text::Family::Monospace),
                    Shaping::Advanced,
                );

                event_loop_proxy.send_event(()).unwrap();
            });
        }

        Self {
            windows: Default::default(),
            context,
            font_system,
            swash_cache,
            widget_buffer,
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<WindowId, Box<dyn Error>> {
        let mut window_attributes = WindowAttributes::default();

        if let Some(token) = event_loop.read_token_from_env() {
            window_attributes = window_attributes.with_activation_token(token);
        }

        let window = event_loop.create_window(window_attributes)?;
        window.set_fullscreen(Some(Fullscreen::Borderless(window.primary_monitor())));
        //window.set_cursor_visible(false);

        let window_state = WindowState::new(self, window)?;
        let window_id = window_state.window.id();

        self.windows.insert(window_id, window_state);

        Ok(window_id)
    }
}

impl ApplicationHandler for Application {
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        for window_state in self.windows.values() {
            window_state.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        event_loop.set_control_flow(ControlFlow::Wait);

        let window = match self.windows.get_mut(&window_id) {
            Some(window) => window,
            None => return,
        };

        match event {
            WindowEvent::Resized(size) => {
                window.resize(size);
            }

            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                window.modifiers = modifiers.state();
            }

            WindowEvent::RedrawRequested => {
                if let Err(err) = window.draw() {
                    println!("Error drawing window: {err}");
                }
            }

            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                let mods = window.modifiers;

                if event.state.is_pressed() {
                    // TODO make keybinding to close configurable
                    if let Key::Character(ch) = event.logical_key.as_ref() {
                        if ch.to_uppercase() == "Q" && mods == ModifiersState::SUPER {
                            event_loop.exit();
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.windows.is_empty() {
            event_loop.exit();
        }
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // We must drop the context here.
        self.context = None;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("failed to create window");
    }
}

enum Background {
    Image {
        image: DynamicImage,
        original: Option<DynamicImage>,
    },
    #[allow(dead_code)]
    SolidColor((u8, u8, u8)),
}

impl Background {
    fn new_image(path: impl Into<String>) -> Result<Self, Box<dyn Error>> {
        let path = path.into();
        let image = ImageReader::open(&path)?.decode()?;

        Ok(Self::Image {
            image,
            original: None,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        // TODO refactor this piece of shit
        if let Self::Image { image, original } = self {
            let img = match original {
                Some(original) => original,
                None => image,
            };

            let resized = img.resize_to_fill(
                size.width,
                size.height,
                image::imageops::FilterType::Nearest,
            );

            if original.is_none() {
                *original = Some(image.clone());
            }

            *image = resized;
        }
    }
}

struct WindowState {
    // Render Surface.
    surface: Surface<DisplayHandle<'static>, Arc<Window>>,

    /// winit Window.
    window: Arc<Window>,

    /// Window Background.
    background: Background,

    /// Window modifiers.
    modifiers: ModifiersState,

    font_system: Arc<Mutex<FontSystem>>,
    swash_cache: Arc<Mutex<SwashCache>>,
    widget_buffer: Arc<Mutex<Buffer>>,
}

impl WindowState {
    fn new(app: &Application, window: Window) -> Result<Self, Box<dyn Error>> {
        let window = Arc::new(window);
        let surface = Surface::new(app.context.as_ref().unwrap(), Arc::clone(&window))?;

        let size = window.inner_size();

        // TODO read it from configuration
        //let background = Background::SolidColor((0, 0, 0));
        let mut background = Background::new_image(
            "/home/koritar/.config/wallpapers/yosemite-valley.jpg".to_string(),
        )?;
        background.resize(size);

        let state = WindowState {
            surface,
            window,
            background,
            modifiers: Default::default(),
            font_system: app.font_system.clone(),
            swash_cache: app.swash_cache.clone(),
            widget_buffer: app.widget_buffer.clone(),
        };

        Ok(state)
    }

    /// Resize the window to the new size.
    fn resize(&mut self, size: PhysicalSize<u32>) {
        let (width, height) = match (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
            (Some(width), Some(height)) => (width, height),
            _ => return,
        };

        self.surface
            .resize(width, height)
            .expect("failed to resize surface");

        self.background.resize(size);

        self.window.request_redraw();
    }

    fn draw(&mut self) -> Result<(), Box<dyn Error>> {
        let mut buffer = self.surface.buffer_mut()?;

        match &self.background {
            Background::Image { image, original: _ } => {
                let width = image.width() as usize;

                for (x, y, pixel) in image.pixels() {
                    // let [red, green, blue, _] = pixel.0;
                    let red = pixel.0[0] as u32;
                    let green = pixel.0[1] as u32;
                    let blue = pixel.0[2] as u32;

                    let color = blue | (green << 8) | (red << 16);

                    buffer[y as usize * width + x as usize] = color;
                }
            }
            Background::SolidColor((r, g, b)) => {
                buffer.fill(u32::from_be_bytes([0x00, *r, *g, *b]));
            }
        }

        let buffer_u8 = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len() * 4)
        };

        let mut pixmap = PixmapMut::from_bytes(
            buffer_u8,
            self.window.inner_size().width,
            self.window.inner_size().height,
        )
        .unwrap();

        let mut paint = Paint {
            anti_alias: false,
            ..Default::default()
        };

        let mut font_system = self.font_system.lock().unwrap();
        let mut swash_cache = self.swash_cache.lock().unwrap();
        let widget_buffer = self.widget_buffer.lock().unwrap();

        let padding_x = 50;
        let padding_y = 100;

        widget_buffer.draw(
            &mut font_system,
            &mut swash_cache,
            Color::rgb(255, 255, 255),
            |x, y, w, h, color| {
                paint.set_color_rgba8(color.b(), color.g(), color.r(), color.a());
                pixmap.fill_rect(
                    Rect::from_xywh(
                        (x + padding_x) as f32,
                        (y + padding_y) as f32,
                        w as f32,
                        h as f32,
                    )
                    .unwrap(),
                    &paint,
                    Transform::identity(),
                    None,
                );
            },
        );

        self.window.pre_present_notify();
        buffer.present()?;
        Ok(())
    }
}
