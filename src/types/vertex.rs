#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

// input vertices in counterclockwise order - ccw is forward remember!
// pub const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 0.0, 0.5] }, // A
//     Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.5, 0.0, 0.5] }, // B
//     Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.5, 0.0, 0.5] }, // C
//     Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] }, // D
//     Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.5, 0.0, 0.5] }, // E
// ];

// // indices buffer, so that we can draw more complex shapes!
// pub const INDICES: &[u16] = &[
//     0, 1, 4,
//     1, 2, 4,
//     2, 3, 4,
// ];

pub const VERTICES: &[Vertex] = &[
    Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.0, 0.5, 0.5] }, // A
    Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.0, 0.5, 0.5] }, // B
    Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.0, 0.5, 0.5] }, // C
    Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.0, 0.5, 0.5] }, // D
    Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.0, 0.5, 0.5] }, // E
];

// indices buffer, so that we can draw more complex shapes!
pub const INDICES: &[u16] = &[
    0, 1, 4,
    1, 2, 4,
    2, 3, 4,
];

// lib.rs
impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress, // defines how wide a vertex will be in memory
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

    pub fn generate_polygon(num_sides: u16, radius: f32) -> (Vec<Vertex>, Vec<u16>) {
        let angle = std::f32::consts::PI * 2.0 / num_sides as f32;
        let vertices = (0..(num_sides * 3))
            .map(|i| {
                let theta = angle * i as f32;
                Vertex {
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
