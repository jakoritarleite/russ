use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use chrono::Timelike;
use cosmic_text::Attrs;
use cosmic_text::Buffer;
use cosmic_text::Color;
use cosmic_text::FontSystem;
use cosmic_text::Metrics;
use cosmic_text::Shaping;
use cosmic_text::SwashCache;
use spin::RwLock;
use tiny_skia::Paint;
use tiny_skia::PixmapMut;
use tiny_skia::Rect;
use tiny_skia::Transform;
use winit::event_loop::EventLoop;
use winit::raw_window_handle::DisplayHandle;
use winit::window::Window;

use crate::render::Drawable;

pub struct Clock {
    buffer: Buffer,
    font_system: FontSystem,
    swash_cache: SwashCache,

    current_time: Arc<RwLock<String>>,
}

impl Clock {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, Box<dyn Error>> {
        // cosmic-text says we should use one per application, but who cares? I don't want to use
        // mutexes so here we go.
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        // TODO how this line height works???
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(150.0, 1.0));
        buffer.set_text(
            &mut font_system,
            &get_time(),
            Attrs::new().family(cosmic_text::Family::Monospace),
            Shaping::Advanced,
        );

        let current_time = Arc::new(RwLock::new(get_time()));

        {
            let event_loop_proxy = event_loop.create_proxy();
            let current_time = current_time.clone();
            //let buffer = buffer.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));

                let mut ctime = current_time.write();
                *ctime = get_time();

                //let mut font_system = font_system.lock().unwrap();
                //let mut buffer = buffer.lock().unwrap();
                //buffer.set_text(
                //    &mut font_system,
                //    &get_time(),
                //    Attrs::new().family(cosmic_text::Family::Monospace),
                //    Shaping::Advanced,
                //);

                event_loop_proxy.send_event(()).unwrap();
            });
        }

        Ok(Self {
            buffer,
            font_system,
            swash_cache,
            current_time,
        })
    }
}

impl Drawable for Clock {
    fn draw(
        &mut self,
        window: &Window,
        buffer: &mut softbuffer::Buffer<DisplayHandle<'static>, Arc<Window>>,
    ) -> Result<(), Box<dyn Error>> {
        self.buffer.set_text(
            &mut self.font_system,
            &self.current_time.read(),
            Attrs::new().family(cosmic_text::Family::Monospace),
            Shaping::Advanced,
        );

        let buffer_u8 = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, buffer.len() * 4)
        };

        let mut pixmap = PixmapMut::from_bytes(
            buffer_u8,
            window.inner_size().width,
            window.inner_size().height,
        )
        .unwrap();

        let mut paint = Paint {
            anti_alias: false,
            ..Default::default()
        };

        let padding_x = 50;
        let padding_y = 100;

        self.buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            // TODO: make this color configurable
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

        Ok(())
    }
}

fn get_time() -> String {
    let dt = chrono::Local::now();
    format!("{}:{}:{}", dt.hour(), dt.minute(), dt.second())
}
