use winit::keyboard::KeyCode;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub scale: f32,
    pub x: f32,
    pub y: f32
}

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_zoom_in: bool,
    is_zoom_out: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_zoom_in: false,
            is_zoom_out: false,
        }
    }

    pub fn handle_key(&mut self, code: KeyCode, is_pressed: bool) -> bool {
        match code {
            KeyCode::KeyW | KeyCode::ArrowUp => {
                self.is_forward_pressed = is_pressed;
                true
            }
            KeyCode::KeyA | KeyCode::ArrowLeft => {
                self.is_left_pressed = is_pressed;
                true
            }
            KeyCode::KeyS | KeyCode::ArrowDown => {
                self.is_backward_pressed = is_pressed;
                true
            }
            KeyCode::KeyD | KeyCode::ArrowRight => {
                self.is_right_pressed = is_pressed;
                true
            }
            KeyCode::KeyQ => {
                self.is_zoom_in = is_pressed;
                true
            }
            KeyCode::KeyE => {
                self.is_zoom_out = is_pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera) {
        if self.is_forward_pressed {
            camera.y -= self.speed / camera.scale;
        }
        if self.is_backward_pressed {
            camera.y += self.speed / camera.scale;
        }
        if self.is_left_pressed {
            camera.x += self.speed / camera.scale;
        }
        if self.is_right_pressed {
            camera.x -= self.speed / camera.scale;
        }
        if self.is_zoom_in {
            camera.scale += self.speed;
        }
        if self.is_zoom_out {
            camera.scale -= self.speed;
        }
    }
}
