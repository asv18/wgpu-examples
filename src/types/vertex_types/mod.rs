pub mod colored_vertex;
pub mod textured_vertex;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}