use cosmic_text::Attrs;
use cosmic_text::AttrsOwned;
use cosmic_text::Buffer;
use cosmic_text::Color;
use cosmic_text::Family;
use cosmic_text::FontSystem;
use cosmic_text::Metrics;
use cosmic_text::Shaping;
use cosmic_text::SwashCache;
use cosmic_text::Weight;
use tiny_skia::Paint;
use tiny_skia::Pixmap;
use tiny_skia::Rect;
use tiny_skia::Transform;
use winit::window::Window;

use crate::config::TextConfig;
use crate::render::DrawError;
use crate::render::Drawable;

use super::Position;
use super::WidgetError;

pub struct Text {
    position: Position,
    font_system: FontSystem,
    swash_cache: SwashCache,
    font_attrs: AttrsOwned,

    data: String,

    buffer: Buffer,
}

impl Text {
    pub fn new(config: TextConfig) -> Result<Self, WidgetError> {
        // cosmic-text says we should use one per application, but who cares? I don't want to use
        // mutexes so here we go.
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        let attrs = Attrs::new()
            .family(
                config
                    .font
                    .font_family
                    .as_ref()
                    .map(|family| Family::Name(family))
                    .unwrap_or(Family::Monospace),
            )
            .weight(Weight(config.font.font_weight));
        let attrs = AttrsOwned::new(attrs);

        // TODO how this line height works???
        let mut buffer = Buffer::new(
            &mut font_system,
            Metrics::new(config.font.font_size, config.font.line_height),
        );
        buffer.set_text(
            &mut font_system,
            &config.text,
            attrs.as_attrs(),
            Shaping::Advanced,
        );

        Ok(Self {
            buffer,
            font_system,
            swash_cache,
            font_attrs: attrs,
            position: config.position,
            data: config.text.clone(),
        })
    }

    pub(super) fn update_data(&mut self, data: String) {
        self.data = data;
    }
}

impl Drawable for Text {
    fn draw(&mut self, window: &Window, buffer: &mut Pixmap) -> Result<(), DrawError> {
        self.buffer.set_text(
            &mut self.font_system,
            &self.data,
            self.font_attrs.as_attrs(),
            Shaping::Advanced,
        );

        let mut paint = Paint {
            anti_alias: true,
            ..Default::default()
        };

        let (padding_x, padding_y) = match self.position {
            Position::Center => {
                let (width, height) = self
                    .buffer
                    .layout_runs()
                    .fold((0.0, 0.0), |(width, height), run| {
                        (run.line_w.max(width), height + 1.0)
                    });

                let height = height * self.buffer.metrics().line_height;

                let centered_x = (window.inner_size().width / 2) - (width / 2.0) as u32;
                let centered_y = (window.inner_size().height / 2) - (height / 2.0) as u32;

                (centered_x as i32, centered_y as i32)
            }

            Position::XY { x, y } => (x as i32, y as i32),

            Position::CenteredX { y } => {
                let width = self
                    .buffer
                    .layout_runs()
                    .fold(0.0, |width, run| run.line_w.max(width));

                let centered_x = (window.inner_size().width / 2) - (width / 2.0) as u32;

                (centered_x as i32, y as i32)
            }

            Position::CenteredY { x } => {
                let height = self.buffer.layout_runs().count();
                let height = height as f32 * self.buffer.metrics().line_height;

                let centered_y = (window.inner_size().height / 2) - (height / 2.0) as u32;

                (x as i32, centered_y as i32)
            }
        };

        self.buffer.draw(
            &mut self.font_system,
            &mut self.swash_cache,
            // TODO: make this color configurable
            Color::rgba(255, 255, 255, 100),
            |x, y, w, h, color| {
                paint.set_color_rgba8(color.r(), color.g(), color.b(), color.a());
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
