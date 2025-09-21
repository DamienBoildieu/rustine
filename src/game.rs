use crate::graphics::CameraController;

// This code should be game specific and not part of the crate
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
