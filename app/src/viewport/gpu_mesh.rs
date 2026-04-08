//! GPU mesh conversion utilities.
//!
//! Converts [`EditMesh`] from the modeling crate into [`GpuMesh`] for the
//! render crate, applying world transforms (position, rotation, scale).

use forge_modeling::half_edge::EditMesh;
use forge_render::vertex::{GpuMesh, Vertex};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Convert an `EditMesh` into a GPU-ready `GpuMesh`, applying the given
/// world-space position, Euler rotation (degrees), and uniform scale.
pub(crate) fn editmesh_to_gpu(
    device: &wgpu::Device,
    mesh: &EditMesh,
    position: Vec3,
    rotation_deg: Vec3,
    scale: Vec3,
    color: [f32; 4],
) -> GpuMesh {
    let (positions, normals, uvs, indices) = mesh.to_triangles();

    // Build model matrix: scale → rotate → translate
    let rot_mat = Mat4::from_euler(
        glam::EulerRot::XYZ,
        rotation_deg.x.to_radians(),
        rotation_deg.y.to_radians(),
        rotation_deg.z.to_radians(),
    );
    let model = Mat4::from_translation(position)
        * rot_mat
        * Mat4::from_scale(scale);

    // Normal matrix (inverse transpose of upper-left 3x3)
    let normal_mat = rot_mat; // for uniform scale, rotation is sufficient

    let vertices: Vec<Vertex> = positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| {
            let world_pos = model.transform_point3(pos);
            let world_normal = normal_mat.transform_vector3(normals[i]).normalize();
            Vertex {
                position: world_pos.to_array(),
                normal: world_normal.to_array(),
                uv: uvs[i].to_array(),
                color,
            }
        })
        .collect();

    let bounds = GpuMesh::compute_aabb(&vertices);

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("mesh_vertex_buffer"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("mesh_index_buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    GpuMesh {
        vertex_buffer,
        index_buffer,
        index_count: indices.len() as u32,
        index_format: wgpu::IndexFormat::Uint32,
        bounds,
    }
}
