use wgpu::{util::DeviceExt, Device};

use super::vertex::Vertex;

pub struct PolygonBuffer {
    // check macro kata to make stuff like this more readable
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub num_indices: u32,
}

impl PolygonBuffer {
    pub fn new(device: &Device, vertices: &[Vertex], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        
        let num_vertices = vertices.len() as u32;

        let num_indices = indices.len() as u32;

        Self { vertex_buffer, index_buffer, num_vertices, num_indices }
    }
}