//! Mesh loading from glTF / GLB files and uploading to the GPU.
//!
//! Each glTF mesh primitive becomes a separate [`GpuMesh`] with its own vertex
//! and index buffers. Use [`load_glb`] for files on disk or
//! [`load_glb_from_bytes`] for in-memory data.

use std::path::Path;

use anyhow::{Context, Result};
use wgpu::util::DeviceExt;

use crate::vertex::{GpuMesh, Vertex};

/// Load all meshes from a GLB/glTF file, uploading vertex and index buffers to the GPU.
///
/// Each primitive in each mesh becomes a separate `GpuMesh`.
pub fn load_glb(
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    path: &Path,
) -> Result<Vec<GpuMesh>> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to read glTF file: {}", path.display()))?;
    load_glb_from_bytes(device, &data)
}

/// Load all meshes from in-memory GLB/glTF data.
pub fn load_glb_from_bytes(device: &wgpu::Device, data: &[u8]) -> Result<Vec<GpuMesh>> {
    let (doc, buffers, _images) =
        gltf::import_slice(data).context("Failed to parse glTF data")?;

    let mut gpu_meshes = Vec::new();

    for mesh in doc.meshes() {
        for primitive in mesh.primitives() {
            let reader =
                primitive.reader(|buf| buffers.get(buf.index()).map(|d| &d[..]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .context("Mesh primitive has no positions")?
                .collect();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|it| it.collect())
                .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|it| it.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let colors: Vec<[f32; 4]> = reader
                .read_colors(0)
                .map(|it| it.into_rgba_f32().collect())
                .unwrap_or_else(|| vec![[1.0, 1.0, 1.0, 1.0]; positions.len()]);

            let vertices: Vec<Vertex> = positions
                .iter()
                .enumerate()
                .map(|(i, &pos)| Vertex {
                    position: pos,
                    normal: normals[i],
                    uv: uvs[i],
                    color: colors[i],
                })
                .collect();

            let indices: Vec<u32> = reader
                .read_indices()
                .context("Mesh primitive has no indices")?
                .into_u32()
                .collect();

            let bounds = GpuMesh::compute_aabb(&vertices);

            let mesh_name = mesh.name().unwrap_or("unnamed");
            let vtx_label = format!("{mesh_name}_vtx");
            let idx_label = format!("{mesh_name}_idx");

            let vertex_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&vtx_label),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

            let index_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&idx_label),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

            gpu_meshes.push(GpuMesh {
                vertex_buffer,
                index_buffer,
                index_count: indices.len() as u32,
                index_format: wgpu::IndexFormat::Uint32,
                bounds,
            });
        }
    }

    if gpu_meshes.is_empty() {
        anyhow::bail!("glTF file contains no mesh primitives");
    }

    Ok(gpu_meshes)
}
