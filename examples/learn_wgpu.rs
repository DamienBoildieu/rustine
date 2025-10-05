//! Implementation of <https://sotrh.github.io/learn-wgpu/> using Rustine engine
//! The content is not exactly matching the tutorial content as it has been adapted to implement challenges in my own way

use winit::event_loop::{ControlFlow, EventLoop};

use rustine::graphics::{CameraController, Renderer};

use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

pub struct Game {
    pub position_color: wgpu::Color,
    pub camera_controller: CameraController,
}

impl Default for Game {
    fn default() -> Self {
        Game {
            position_color: wgpu::Color {
                r: 0.,
                g: 0.,
                b: 0.5,
                a: 1.,
            },
            camera_controller: CameraController::new(0.02),
        }
    }
}

#[derive(Default)]
pub struct WindowApp {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    game: Game,
}

impl WindowApp {
    fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            // (KeyCode::Space, true) => self.game.swap_texture = !self.game.swap_texture,
            _ => {}
        }

        let renderer = match &mut self.renderer {
            Some(canvas) => canvas,
            None => return,
        };

        renderer.camera_controller.process_events(code, is_pressed);
    }

    fn handle_mouse_moved(&mut self, _: DeviceId, position: &PhysicalPosition<f64>) {
        let renderer = self.renderer.as_mut().unwrap();
        self.game.position_color.r = position.x / renderer.config.width as f64;
        self.game.position_color.g = position.y / renderer.config.height as f64;
    }

    fn handle_mouse_input(&mut self, _: DeviceId, button: MouseButton, is_pressed: bool) {
        match (button, is_pressed) {
            (MouseButton::Left, true) => {
                let renderer = self.renderer.as_mut().unwrap();
                renderer.update_light_color(self.game.position_color);
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for WindowApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes().with_title("Rustine");

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        self.renderer = Some(pollster::block_on(Renderer::new(window)).unwrap());
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let window = match &self.window {
            Some(winow_arc) => winow_arc.as_ref(),
            None => return,
        };

        let renderer = match &mut self.renderer {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                // Could use if let instead of unwwrap
                renderer.resize(size.width, size.height);
            }
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.

                renderer.update_light_position();
                renderer.update_camera();
                // renderer.update_model();
                window.request_redraw();
                match renderer.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = window.inner_size();
                        renderer.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => self.handle_key(event_loop, code, key_state.is_pressed()),
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => self.handle_mouse_moved(device_id, &position),
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => self.handle_mouse_input(device_id, button, state.is_pressed()),
            _ => (),
        }
    }
}

fn main() -> Result<(), winit::error::EventLoopError> {
    env_logger::init();

    let event_loop = EventLoop::new()?;

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = WindowApp::default();
    event_loop.run_app(&mut app)
}
