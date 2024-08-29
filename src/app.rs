use std::collections::HashMap;
use std::error::Error;

use softbuffer::Context;
use winit::application::ApplicationHandler;
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
use winit::window::WindowAttributes;
use winit::window::WindowId;

use crate::background::Background;
use crate::render::Drawable;
use crate::widget::clock::Clock;
use crate::window::WindowState;

pub struct Application {
    pub(crate) windows: HashMap<WindowId, WindowState>,

    // Drawing context.
    pub(crate) context: Option<Context<DisplayHandle<'static>>>,

    pub(crate) background: Background,
    clock_widget: Clock,
}

impl Application {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
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

        // TODO read it from configuration
        //let background = Background::SolidColor((0, 0, 0));
        let background = Background::new_image(
            "/home/koritar/.config/wallpapers/yosemite-valley.jpg".to_string(),
        )
        .unwrap();

        let clock_widget = Clock::new(event_loop).unwrap();

        Self {
            windows: Default::default(),
            context,
            background,
            clock_widget,
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
                let clock_widget: &mut dyn Drawable = &mut self.clock_widget;
                let drawables = vec![background, clock_widget];

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

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // We must drop the context here.
        self.context = None;
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("failed to create window");
    }
}
