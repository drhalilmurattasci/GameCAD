//! Light types and GPU uniform packing.
//!
//! Supports directional, point, and spot lights. [`LightSet`] collects all
//! lights for a frame and packs them into a [`LightUniform`] for the GPU.
//! [`LightsBuffer`] owns the wgpu buffer and bind group.

use bytemuck::{Pod, Zeroable};
use forge_core::math::Color;
use glam::Vec3;
use wgpu::util::DeviceExt;

/// Maximum number of point lights supported per draw call.
pub const MAX_POINT_LIGHTS: usize = 64;

/// Maximum number of spot lights supported per draw call.
pub const MAX_SPOT_LIGHTS: usize = 32;

/// A directional light (e.g. the sun).
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
}

impl Default for DirectionalLight {
    fn default() -> Self {
        Self {
            direction: Vec3::new(-0.3, -1.0, -0.5).normalize(),
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
}

/// A point light with attenuation.
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
}

impl Default for PointLight {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            color: Color::WHITE,
            intensity: 1.0,
            radius: 10.0,
        }
    }
}

/// A spot light with cone angles.
#[derive(Debug, Clone, Copy)]
pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub color: Color,
    pub intensity: f32,
    pub radius: f32,
    /// Inner cone angle in radians (full intensity).
    pub inner_cutoff: f32,
    /// Outer cone angle in radians (falloff to zero).
    pub outer_cutoff: f32,
}

impl Default for SpotLight {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            direction: Vec3::NEG_Y,
            color: Color::WHITE,
            intensity: 1.0,
            radius: 10.0,
            inner_cutoff: 0.3,
            outer_cutoff: 0.5,
        }
    }
}

/// Collection of lights in a scene, ready for GPU upload.
#[derive(Debug, Clone)]
pub struct LightSet {
    pub directional: DirectionalLight,
    pub point_lights: Vec<PointLight>,
    pub spot_lights: Vec<SpotLight>,
    pub ambient_color: Color,
    pub ambient_intensity: f32,
}

impl Default for LightSet {
    fn default() -> Self {
        Self {
            directional: DirectionalLight::default(),
            point_lights: Vec::new(),
            spot_lights: Vec::new(),
            ambient_color: Color::WHITE,
            ambient_intensity: 0.15,
        }
    }
}

impl LightSet {
    /// Pack into a GPU-ready uniform struct.
    pub fn to_uniform(&self) -> LightUniform {
        let mut point_positions = [[0.0f32; 4]; MAX_POINT_LIGHTS];
        let mut point_colors = [[0.0f32; 4]; MAX_POINT_LIGHTS];

        for (i, pl) in self.point_lights.iter().take(MAX_POINT_LIGHTS).enumerate() {
            point_positions[i] = [pl.position.x, pl.position.y, pl.position.z, pl.radius];
            let c = pl.color.to_linear();
            point_colors[i] = [
                c[0] * pl.intensity,
                c[1] * pl.intensity,
                c[2] * pl.intensity,
                1.0,
            ];
        }

        let mut spot_pos_inner = [[0.0f32; 4]; MAX_SPOT_LIGHTS];
        let mut spot_dir_outer = [[0.0f32; 4]; MAX_SPOT_LIGHTS];
        let mut spot_colors = [[0.0f32; 4]; MAX_SPOT_LIGHTS];

        for (i, sl) in self.spot_lights.iter().take(MAX_SPOT_LIGHTS).enumerate() {
            spot_pos_inner[i] = [
                sl.position.x,
                sl.position.y,
                sl.position.z,
                sl.inner_cutoff.cos(),
            ];
            spot_dir_outer[i] = [
                sl.direction.x,
                sl.direction.y,
                sl.direction.z,
                sl.outer_cutoff.cos(),
            ];
            let c = sl.color.to_linear();
            spot_colors[i] = [
                c[0] * sl.intensity,
                c[1] * sl.intensity,
                c[2] * sl.intensity,
                sl.radius,
            ];
        }

        let dc = self.directional.color.to_linear();
        let ac = self.ambient_color.to_linear();

        LightUniform {
            dir_direction: [
                self.directional.direction.x,
                self.directional.direction.y,
                self.directional.direction.z,
                0.0,
            ],
            dir_color: [
                dc[0] * self.directional.intensity,
                dc[1] * self.directional.intensity,
                dc[2] * self.directional.intensity,
                1.0,
            ],
            ambient: [
                ac[0] * self.ambient_intensity,
                ac[1] * self.ambient_intensity,
                ac[2] * self.ambient_intensity,
                1.0,
            ],
            num_point_lights: self.point_lights.len().min(MAX_POINT_LIGHTS) as u32,
            num_spot_lights: self.spot_lights.len().min(MAX_SPOT_LIGHTS) as u32,
            _pad: [0; 2],
            point_positions,
            point_colors,
            spot_pos_inner,
            spot_dir_outer,
            spot_colors,
        }
    }
}

/// GPU-side light data (std140).
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightUniform {
    // Directional light
    pub dir_direction: [f32; 4],
    pub dir_color: [f32; 4],
    // Ambient
    pub ambient: [f32; 4],
    // Counts
    pub num_point_lights: u32,
    pub num_spot_lights: u32,
    pub _pad: [u32; 2],
    // Point lights
    pub point_positions: [[f32; 4]; MAX_POINT_LIGHTS],
    pub point_colors: [[f32; 4]; MAX_POINT_LIGHTS],
    // Spot lights
    pub spot_pos_inner: [[f32; 4]; MAX_SPOT_LIGHTS],
    pub spot_dir_outer: [[f32; 4]; MAX_SPOT_LIGHTS],
    pub spot_colors: [[f32; 4]; MAX_SPOT_LIGHTS],
}

/// A GPU buffer holding light uniform data.
pub struct LightsBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl LightsBuffer {
    /// Create a new lights buffer with initial data.
    pub fn new(device: &wgpu::Device, lights: &LightSet) -> Self {
        let uniform = lights.to_uniform();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("lights_uniform"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("lights_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("lights_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    /// Upload new light data to the GPU.
    pub fn update(&self, queue: &wgpu::Queue, lights: &LightSet) {
        let uniform = lights.to_uniform();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}
