use std::process::{self};

use application::Application;
use window::Window;
use winit::{
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowAttributes,
};

pub mod application;
pub mod camera;
pub mod error;
pub mod render;
pub mod window;
pub mod world2;

#[macro_export]
macro_rules! asset {
    ($path:literal) => {
        concat!(env!("CARGO_MANIFEST_DIR"), "/..", "/assets/", $path)
    };
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().expect("failed to create event loop");

    let mut window = Window::new(|event_loop: &ActiveEventLoop| {
        let window = event_loop
            .create_window(WindowAttributes::default())
            .expect("failed to create window");

        match pollster::block_on(Application::new(window)) {
            Ok(application) => application,
            Err(err) => {
                eprintln!("{err}");
                process::exit(1)
            }
        }
    });

    match event_loop.run_app(&mut window) {
        Ok(..) => {}
        Err(err) => eprintln!("event loop error: {err:?}"),
    }
}
