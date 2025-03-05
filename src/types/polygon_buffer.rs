use std::marker::PhantomData;

use wgpu::{util::DeviceExt, Device};

use super::vertex_types::Vertex;

pub struct PolygonBuffer<T: bytemuck::Pod + bytemuck::Zeroable + Vertex> {
    // check macro kata to make stuff like this more readable
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub _num_vertices: u32,
    pub num_indices: u32,
    _marker: PhantomData<T>,
}

impl<T: bytemuck::Pod + bytemuck::Zeroable + Vertex> PolygonBuffer<T> {
    pub fn new(device: &Device, vertices: &[T], indices: &[u16]) -> Self {
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
        
        let _num_vertices = vertices.len() as u32;

        let num_indices = indices.len() as u32;

        Self { vertex_buffer, index_buffer, _num_vertices, num_indices, _marker: PhantomData }
    }
}