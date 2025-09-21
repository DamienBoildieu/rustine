
// Use nalgebra points?

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub position: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    pub color: [f32; 3],
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

impl LightUniform {
    pub fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            position,
            _padding: 0,
            color,
            _padding2: 0,
        }
    }
}

impl Default for LightUniform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            _padding: 0,
            color: [1.0; 3],
            _padding2: 0,
        }
    }
}
