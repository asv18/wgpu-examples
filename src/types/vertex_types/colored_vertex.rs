use super::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColoredVertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex for ColoredVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColoredVertex>() as wgpu::BufferAddress, // defines how wide a vertex will be in memory
            step_mode: wgpu::VertexStepMode::Vertex, // tells the pipeline whether to handle the vertices in a per-vertex or per-instance case
            attributes: &[ // describe the individual parts of the vertex
                wgpu::VertexAttribute {
                    offset: 0, // offset in bytes until an attribute starts
                    shader_location: 0, // tells the shader where to store this attribute at
                    format: wgpu::VertexFormat::Float32x3, // tells the shader what the shape of the attribute is
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}

// lib.rs
impl ColoredVertex {
    pub fn generate_polygon(num_sides: u16, radius: f32) -> (Vec<ColoredVertex>, Vec<u16>) {
        let angle = std::f32::consts::PI * 2.0 / num_sides as f32;
        let vertices = (0..(num_sides * 3))
            .map(|i| {
                let theta = angle * i as f32;
                ColoredVertex {
                    position: [radius * theta.sin(), radius * theta.cos(), 0.0],
                    color: [radius * theta.sin(), radius * theta.cos(), 1.0],
                }
                // [(1.0 + theta.cos()) / 2.0, (1.0 + theta.sin()) / 2.0, 1.0]
            })
            .collect();

            let num_triangles = (num_sides * 3) - 2;
            let indices = (1u16..num_triangles + 1)
                .into_iter()
                .flat_map(|i| vec![0, i + 1, i])
                .collect();

        (vertices, indices)
    }
}
