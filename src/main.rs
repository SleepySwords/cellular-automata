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
        winit::event::Event::WindowEvent { event, window_id }
            if window_id == renderer.window().id() =>
        {
            if !renderer.input(&event) {
                match event {
                    WindowEvent::CloseRequested => event_loop_window_target.exit(),
                    WindowEvent::Resized(new_size) => {
                        surface_configured = true;
                        renderer.resize(new_size)
                    }
                    WindowEvent::RedrawRequested => match renderer.render() {
                        Ok(_) => {}
                        Err(SurfaceError::Lost) => renderer.resize(renderer.size),
                        _ => {
                            return;
                        }
                    },
                    _ => {}
                }
            }
        }
        Event::AboutToWait => {
            renderer.window().request_redraw();
        }
        _ => {}
    })?;

    return Ok(());
}
