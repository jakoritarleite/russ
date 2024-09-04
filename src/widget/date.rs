use std::sync::Arc;
use std::time::Duration;

use spin::RwLock;
use tiny_skia::Pixmap;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::config::DateConfig;
use crate::config::TextConfig;
use crate::render::DrawError;
use crate::render::Drawable;

use super::text::Text;
use super::WidgetError;

pub struct Date {
    text_widget: Text,
    current_time: Arc<RwLock<String>>,
}

impl Date {
    pub fn new(event_loop: &EventLoop<()>, config: DateConfig) -> Result<Self, WidgetError> {
        let text_config = TextConfig {
            text: get_date(&config.format),
            position: config.position,
            font: config.font,
        };
        let widget = Text::new(text_config)?;

        let current_time = Arc::new(RwLock::new(get_date(&config.format)));

        {
            let event_loop_proxy = event_loop.create_proxy();
            let current_time = current_time.clone();

            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(60));

                *current_time.write() = get_date(&config.format);
                event_loop_proxy.send_event(()).unwrap();
            });
        }

        Ok(Self {
            text_widget: widget,
            current_time,
        })
    }
}

impl Drawable for Date {
    fn draw(&mut self, window: &Window, buffer: &mut Pixmap) -> Result<(), DrawError> {
        self.text_widget
            .update_data(self.current_time.read().to_string());
        self.text_widget.draw(window, buffer)
    }
}

fn get_date(format: &str) -> String {
    let dt = chrono::Local::now();

    format!("{}", dt.format(format))
}
