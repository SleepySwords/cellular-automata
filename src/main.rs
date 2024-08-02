mod renderer;

use std::error::Error;

use log::info;
use renderer::Renderer;
use wgpu::SurfaceError;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    info!("ok");

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let window = Window::new(&event_loop).unwrap();

    let mut renderer = Renderer::new(&window).await;
    let mut surface_configured = false;

    event_loop.run(|event, event_loop_window_target| match event {
        winit::event::Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == renderer.window().id() => event_loop_window_target.exit(),
        winit::event::Event::WindowEvent {
            event: WindowEvent::Resized(new_size),
            window_id,
        } if window_id == renderer.window().id() => {
            surface_configured = true;
            renderer.resize(new_size)
        }
        winit::event::Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            window_id,
        } if window_id == renderer.window().id() => {
            match renderer.render() {
                Ok(_) => {}
                Err(SurfaceError::Lost) => renderer.resize(renderer.size),
                _ => {
                    return;
                }
            }
        }
        Event::AboutToWait => {
            renderer.window().request_redraw();
        }
        winit::event::Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            window_id,
        } if window_id == renderer.window().id() => renderer.input(position),
        _ => {}
    })?;

    return Ok(());
}
