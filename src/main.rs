pub mod camera;
mod renderer;
pub mod texture;
pub mod vertex;

use std::{error::Error, sync::Arc};

use log::info;
use renderer::State;
use wgpu::SurfaceError;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, Event, KeyEvent, MouseScrollDelta, TouchPhase, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    info!("ok");

    let event_loop = EventLoop::with_user_event().build()?;
    // event_loop.set_control_flow(ControlFlow::Wait);

    event_loop.run_app(&mut App::new());

    return Ok(());
}

struct App {
    state: Option<State>,
}

impl App {
    fn new() -> App {
        return App { state: None };
    }
}

impl ApplicationHandler<App> for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window_attributes = Window::default_attributes();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(State::new(window)));
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        if !state.input(&event) {
            match event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(code),
                            state: key_state,
                            ..
                        },
                    ..
                } => match (code, key_state.is_pressed()) {
                    (KeyCode::Escape, true) => event_loop.exit(),
                    _ => {}
                },
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(new_size) => state.resize(new_size),
                WindowEvent::RedrawRequested => {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(SurfaceError::Lost) => state.resize(state.size),
                        _ => {
                            return;
                        }
                    }
                }
                _ => {}
            }
        } else {
            state.window().request_redraw();
        }
    }
}
