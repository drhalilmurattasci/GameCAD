//! Vertex layout and GPU mesh types.
//!
//! Defines the [`Vertex`] struct matching the WGSL shader input layout, and
//! [`GpuMesh`] which owns GPU-side vertex/index buffers plus bounding info.

use bytemuck::{Pod, Zeroable};
use forge_core::math::AABB;
use glam::Vec3;

/// A single mesh vertex with position, normal, UV, and color.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex {
    /// Returns the `wgpu::VertexBufferLayout` describing this vertex.
    pub fn buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        const ATTRS: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
            0 => Float32x3,  // position
            1 => Float32x3,  // normal
            2 => Float32x2,  // uv
            3 => Float32x4,  // color
        ];

        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: ATTRS,
        }
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [0.0; 2],
            color: [1.0; 4],
        }
    }
}

/// A mesh whose vertex and index buffers live on the GPU, with bounding info.
pub struct GpuMesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
    pub index_format: wgpu::IndexFormat,
    pub bounds: AABB,
}

impl GpuMesh {
    /// Compute an AABB from a slice of vertices.
    ///
    /// Returns a zero-sized AABB at the origin if the slice is empty.
    pub fn compute_aabb(vertices: &[Vertex]) -> AABB {
        if vertices.is_empty() {
            return AABB::new(Vec3::ZERO, Vec3::ZERO);
        }
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);
        for v in vertices {
            let p = Vec3::from(v.position);
            min = min.min(p);
            max = max.max(p);
        }
        AABB::new(min, max)
    }
}
