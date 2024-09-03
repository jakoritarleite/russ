use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use chrono::Timelike;
use spin::RwLock;
use tiny_skia::Pixmap;
use winit::event_loop::EventLoop;
use winit::window::Window;

use crate::config::FontConfig;
use crate::config::Position;
use crate::render::Drawable;

use super::text::Text;

pub struct Clock {
    text_widget: Text,
    current_time: Arc<RwLock<String>>,
}

impl Clock {
    pub fn new(
        event_loop: &EventLoop<()>,
        position: Position,
        show_seconds: bool,
        font_config: &FontConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let widget = Text::new(get_time(show_seconds), position, font_config)?;

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
            text_widget: widget,
            current_time,
        })
    }
}

impl Drawable for Clock {
    fn draw(&mut self, window: &Window, buffer: &mut Pixmap) -> Result<(), Box<dyn Error>> {
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
