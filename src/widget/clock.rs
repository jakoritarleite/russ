use std::sync::Arc;
use std::time::Duration;

use chrono::Timelike;
use spin::RwLock;
use tiny_skia::Pixmap;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::config::ClockConfig;
use crate::config::TextConfig;
use crate::render::DrawError;
use crate::render::Drawable;

use super::text::Text;
use super::WidgetError;

pub struct Clock {
    text_widget: Text,
    current_time: Arc<RwLock<String>>,
}

impl Clock {
    pub fn new(event_loop: &EventLoop<()>, config: ClockConfig) -> Result<Self, WidgetError> {
        let text_config = TextConfig {
            text: get_time(config.show_seconds),
            position: config.position,
            font: config.font,
        };
        let widget = Text::new(text_config)?;

        let current_time = Arc::new(RwLock::new(get_time(config.show_seconds)));

        {
            let event_loop_proxy = event_loop.create_proxy();
            let current_time = current_time.clone();
            let show_seconds = config.show_seconds;

            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_secs(1));

                *current_time.write() = get_time(show_seconds);
                event_loop_proxy.send_event(()).unwrap();
            });
        }

        Ok(Self {
            text_widget: widget,
            current_time,
        })
    }
}

impl Drawable for Clock {
    fn draw(&mut self, window: &Window, buffer: &mut Pixmap) -> Result<(), DrawError> {
        self.text_widget
            .update_data(self.current_time.read().to_string());
        self.text_widget.draw(window, buffer)
    }
}

fn get_time(show_seconds: bool) -> String {
    let dt = chrono::Local::now();

    match show_seconds {
        true => format!("{:0<2}:{:0<2}:{:0<2}", dt.hour(), dt.minute(), dt.second()),
        false => format!("{:0<2}:{:0<2}", dt.hour(), dt.minute()),
    }
}
