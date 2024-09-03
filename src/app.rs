use std::borrow::BorrowMut;
use std::error::Error;

use thiserror::Error;
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
use crate::background::BackgroundConversionError;
use crate::config;
use crate::config::ConfigError;
use crate::config::Configuration;
use crate::render::DrawError;
use crate::render::Drawable;
use crate::widget::clock::Clock;
use crate::widget::date::Date;
use crate::widget::text::Text;
use crate::widget::WidgetError;
use crate::window::WindowState;

pub struct Application {
    window: Option<WindowState>,

    background: Background,
    widgets: Vec<Box<dyn Drawable>>,
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>) -> Result<Self, ApplicationError> {
        let config = Configuration::new()?;

        let background = (&config.background).try_into()?;

        fn cast_box<W: Drawable + 'static>(widget: W) -> Box<dyn Drawable> {
            let widget: Box<dyn Drawable> = Box::new(widget);
            widget
        }

        let widgets: Vec<Box<dyn Drawable>> = config
            .widgets
            .into_iter()
            .map(|widget| match widget {
                config::Widget::Clock(config) => Clock::new(event_loop, config).map(cast_box),

                config::Widget::Text(config) => Text::new(config).map(cast_box),

                config::Widget::Date(config) => Date::new(event_loop, config).map(cast_box),
            })
            .collect::<Result<Vec<Box<_>>, WidgetError>>()?;

        Ok(Self {
            window: None,
            background,
            widgets,
        })
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

        self.window = Some(window_state);

        Ok(window_id)
    }
}

impl ApplicationHandler for Application {
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        if let Some(ref state) = self.window {
            state.window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: winit::event::WindowEvent,
    ) {
        event_loop.set_control_flow(ControlFlow::Wait);

        let window = match self.window {
            Some(ref mut window) => window,
            None => return,
        };

        match event {
            WindowEvent::Resized(size) => {
                self.background.resize(size);
                window.resize(size);
            }

            WindowEvent::CloseRequested => {
                self.window = None;
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
        if self.window.is_none() {
            event_loop.exit();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("failed to create window");
    }
}

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("An error ocurred when parsing the configuration file: {0}")]
    Config(#[from] ConfigError),

    #[error("An error occurred when converting the background: {0}")]
    BackgroundConversion(#[from] BackgroundConversionError),

    #[error("An error ocurrend creating the widget: {0}")]
    Widget(#[from] WidgetError),

    #[error("An error ocurred when drawing: {0}")]
    Draw(#[from] DrawError),
}
