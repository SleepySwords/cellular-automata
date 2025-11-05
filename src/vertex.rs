use wgpu::{VertexAttribute, VertexBufferLayout};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub(crate) position: [f32; 3],
    pub(crate) color: [f32; 3],
    pub(crate) tex_coords: [f32; 2],
}

impl Vertex {
    const ATTR: [VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2];
    pub fn desc() -> VertexBufferLayout<'static> {
        return VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTR,
            // attributes: &[
            //     VertexAttribute {
            //         format: wgpu::VertexFormat::Float32x3,
            //         offset: 0,
            //         shader_location: 0,
            //     },
            //     VertexAttribute {
            //         format: wgpu::VertexFormat::Float32x3,
            //         offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
            //         shader_location: 1,
            //     },
            //     VertexAttribute {
            //         format: wgpu::VertexFormat::Float32x2,
            //         offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 3]>()) as wgpu::BufferAddress,
            //         shader_location: 2,
            //     },
            // ],
        };
    }
}
