use russ::Application;
use std::error::Error;
use winit::event_loop::EventLoop;

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    let mut app = match Application::new(&event_loop) {
        Ok(app) => app,
        Err(error) => {
            eprintln!("{error:?}");
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    Ok(event_loop.run_app(&mut app)?)
}
