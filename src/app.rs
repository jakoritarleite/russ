use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::error::Error;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::keyboard::Key;
use winit::keyboard::ModifiersState;
use winit::platform::startup_notify::EventLoopExtStartupNotify;
use winit::platform::startup_notify::WindowAttributesExtStartupNotify;
use winit::window::Fullscreen;
use winit::window::WindowAttributes;
use winit::window::WindowId;

use crate::background::Background;
use crate::config::Configuration;
use crate::render::Drawable;
use crate::widget::clock::Clock;
use crate::window::WindowState;

pub struct Application {
    windows: HashMap<WindowId, WindowState>,

    background: Background,
    widgets: Vec<Box<dyn Drawable>>,
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        let config = Configuration::new().unwrap();

        let background = (&config.background).try_into().unwrap();

        let widgets: Vec<Box<dyn Drawable>> = config
            .widgets
            .iter()
            .map(|widget| match widget {
                crate::config::Widget::Clock {
                    position,
                    font_size,
                    line_height,
                    show_seconds,
                } => {
                    let clock = Clock::new(
                        event_loop,
                        *position,
                        *font_size,
                        *line_height,
                        *show_seconds,
                    )
                    .unwrap();
                    let clock: Box<dyn Drawable> = Box::new(clock);
                    clock
                }
            })
            .collect();

        Self {
            windows: Default::default(),
            background,
            widgets,
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

        let window_state = WindowState::new(window)?;
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
                self.background.resize(size);
                window.resize(size);
            }

            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
            }

            WindowEvent::ModifiersChanged(modifiers) => {
                window.modifiers = modifiers.state();
            }

            WindowEvent::RedrawRequested => {
                let background: &mut dyn Drawable = &mut self.background;

                let drawables = self.widgets.iter_mut().map(|widget| {
                    let widget: &mut dyn Drawable = widget.borrow_mut();
                    widget
                });

                let drawables = [background].into_iter().chain(drawables).collect();

                if let Err(err) = window.draw(drawables) {
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

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("failed to create window");
    }
}
