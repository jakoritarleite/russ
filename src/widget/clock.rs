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
use tiny_skia::Pixmap;
use tiny_skia::Rect;
use tiny_skia::Transform;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::config::Position;
use crate::render::Drawable;

pub struct Clock {
    buffer: Buffer,
    font_system: FontSystem,
    swash_cache: SwashCache,

    current_time: Arc<RwLock<String>>,

    position: Position,
}

impl Clock {
    pub fn new(
        event_loop: &EventLoop<()>,
        position: Position,
        font_size: f32,
        line_height: f32,
        show_seconds: bool,
    ) -> Result<Self, Box<dyn Error>> {
        // cosmic-text says we should use one per application, but who cares? I don't want to use
        // mutexes so here we go.
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        // TODO how this line height works???
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(font_size, line_height));
        buffer.set_text(
            &mut font_system,
            &get_time(show_seconds),
            Attrs::new().family(cosmic_text::Family::Monospace),
            Shaping::Advanced,
        );

        let current_time = Arc::new(RwLock::new(get_time(show_seconds)));

        {
            let event_loop_proxy = event_loop.create_proxy();
            let current_time = current_time.clone();

            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));

                *current_time.write() = get_time(show_seconds);
                event_loop_proxy.send_event(()).unwrap();
            });
        }

        Ok(Self {
            buffer,
            font_system,
            swash_cache,
            current_time,
            position,
        })
    }
}

impl Drawable for Clock {
    fn draw(&mut self, window: &Window, buffer: &mut Pixmap) -> Result<(), Box<dyn Error>> {
        self.buffer.set_text(
            &mut self.font_system,
            &self.current_time.read(),
            Attrs::new().family(cosmic_text::Family::Monospace),
            Shaping::Advanced,
        );

        let mut paint = Paint {
            anti_alias: false,
            ..Default::default()
        };

        let (width, height) = match self.position {
            Position::Center => self
                .buffer
                .layout_runs()
                .fold((0.0, 0.0), |(width, height), run| {
                    (run.line_w.max(width), height + 1.0)
                }),

            Position::XY { .. } => (0.0, 0.0),
        };

        let height = height * self.buffer.metrics().line_height;

        self.buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            // TODO: make this color configurable
            Color::rgb(255, 255, 255),
            |x, y, w, h, color| {
                let (padding_x, padding_y) = match self.position {
                    Position::Center => {
                        let centered_x = (window.inner_size().width / 2) - (width / 2.0) as u32;
                        let centered_y = (window.inner_size().height / 2) - (height / 2.0) as u32;

                        (centered_x as i32, centered_y as i32)
                    }
                    Position::XY { x, y } => (x as i32, y as i32),
                };

                paint.set_color_rgba8(color.b(), color.g(), color.r(), color.a());
                buffer.fill_rect(
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

fn get_time(show_seconds: bool) -> String {
    let dt = chrono::Local::now();

    match show_seconds {
        true => format!("{:0<2}:{:0<2}:{:0<2}", dt.hour(), dt.minute(), dt.second()),
        false => format!("{:0<2}:{:0<2}", dt.hour(), dt.minute()),
    }
}
