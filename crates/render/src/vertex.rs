//! Vertex layout and GPU mesh types.
//!
//! Defines the [`Vertex`] struct matching the WGSL shader input layout, and
//! [`GpuMesh`] which owns GPU-side vertex/index buffers plus bounding info.
//!
//! # Examples
//!
//! ```
//! use render::vertex::Vertex;
//!
//! let v = Vertex::default();
//! assert_eq!(v.normal, [0.0, 1.0, 0.0]); // default normal is up
//! ```

use std::fmt;

use bytemuck::{Pod, Zeroable};
use forge_core::math::AABB;
use glam::Vec3;

/// A single mesh vertex with position, normal, UV, and color.
///
/// The layout matches the WGSL `VertexInput` struct used in all mesh shaders:
/// - `@location(0)` position: `vec3<f32>`
/// - `@location(1)` normal: `vec3<f32>`
/// - `@location(2)` uv: `vec2<f32>`
/// - `@location(3)` color: `vec4<f32>`
///
/// # Examples
///
/// ```
/// use render::vertex::Vertex;
///
/// let v = Vertex {
///     position: [1.0, 2.0, 3.0],
///     normal: [0.0, 1.0, 0.0],
///     uv: [0.0, 0.0],
///     color: [1.0, 1.0, 1.0, 1.0],
/// };
/// assert_eq!(v.position[0], 1.0);
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    /// Vertex position in object space.
    pub position: [f32; 3],
    /// Surface normal (unit length).
    pub normal: [f32; 3],
    /// Texture coordinates.
    pub uv: [f32; 2],
    /// Per-vertex color (linear RGBA).
    pub color: [f32; 4],
}

impl Vertex {
    /// Returns the `wgpu::VertexBufferLayout` describing this vertex.
    ///
    /// The layout corresponds to the `VertexInput` struct in all mesh WGSL shaders.
    #[inline]
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

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Vertex(pos: [{:.3}, {:.3}, {:.3}])",
            self.position[0], self.position[1], self.position[2],
        )
    }
}

/// A mesh whose vertex and index buffers live on the GPU, with bounding info.
pub struct GpuMesh {
    /// GPU vertex buffer.
    pub vertex_buffer: wgpu::Buffer,
    /// GPU index buffer.
    pub index_buffer: wgpu::Buffer,
    /// Number of indices to draw.
    pub index_count: u32,
    /// Index element format (u16 or u32).
    pub index_format: wgpu::IndexFormat,
    /// Axis-aligned bounding box of the mesh.
    pub bounds: AABB,
}

impl GpuMesh {
    /// Compute an AABB from a slice of vertices.
    ///
    /// Returns a zero-sized AABB at the origin if the slice is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use render::vertex::{GpuMesh, Vertex};
    /// use glam::Vec3;
    ///
    /// let verts = vec![
    ///     Vertex { position: [1.0, 0.0, 0.0], ..Vertex::default() },
    ///     Vertex { position: [-1.0, 2.0, 3.0], ..Vertex::default() },
    /// ];
    /// let aabb = GpuMesh::compute_aabb(&verts);
    /// assert_eq!(aabb.min, Vec3::new(-1.0, 0.0, 0.0));
    /// assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vertex_default() {
        let v = Vertex::default();
        assert_eq!(v.position, [0.0; 3]);
        assert_eq!(v.normal, [0.0, 1.0, 0.0]);
        assert_eq!(v.color, [1.0; 4]);
    }

    #[test]
    fn vertex_size_matches_layout() {
        // position(12) + normal(12) + uv(8) + color(16) = 48 bytes
        assert_eq!(std::mem::size_of::<Vertex>(), 48);
    }

    #[test]
    fn compute_aabb_empty() {
        let aabb = GpuMesh::compute_aabb(&[]);
        assert_eq!(aabb.min, Vec3::ZERO);
        assert_eq!(aabb.max, Vec3::ZERO);
    }

    #[test]
    fn compute_aabb_single_vertex() {
        let v = Vertex {
            position: [1.0, 2.0, 3.0],
            ..Vertex::default()
        };
        let aabb = GpuMesh::compute_aabb(&[v]);
        assert_eq!(aabb.min, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn compute_aabb_multiple() {
        let verts = vec![
            Vertex {
                position: [1.0, -1.0, 0.0],
                ..Vertex::default()
            },
            Vertex {
                position: [-2.0, 3.0, 5.0],
                ..Vertex::default()
            },
            Vertex {
                position: [0.0, 0.0, -1.0],
                ..Vertex::default()
            },
        ];
        let aabb = GpuMesh::compute_aabb(&verts);
        assert_eq!(aabb.min, Vec3::new(-2.0, -1.0, -1.0));
        assert_eq!(aabb.max, Vec3::new(1.0, 3.0, 5.0));
    }

    #[test]
    fn vertex_display() {
        let v = Vertex::default();
        let s = format!("{v}");
        assert!(s.contains("Vertex"));
    }

    #[test]
    fn buffer_layout_has_four_attributes() {
        let layout = Vertex::buffer_layout();
        assert_eq!(layout.attributes.len(), 4);
        assert_eq!(layout.array_stride, 48);
    }
}
