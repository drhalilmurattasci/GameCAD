//! Light types and GPU uniform packing.
//!
//! Supports directional, point, and spot lights. [`LightSet`] collects all
//! lights for a frame and packs them into a [`LightUniform`] for the GPU.
//! [`LightsBuffer`] owns the wgpu buffer and bind group.
//!
//! # Examples
//!
//! ```
//! use render::lights::{LightSet, DirectionalLight, PointLight, MAX_POINT_LIGHTS};
//!
//! let mut lights = LightSet::default();
//! lights.point_lights.push(PointLight::default());
//! let uniform = lights.to_uniform();
//! assert_eq!(uniform.num_point_lights, 1);
//! ```

use std::fmt;

use bytemuck::{Pod, Zeroable};
use forge_core::math::Color;
use glam::Vec3;
use wgpu::util::DeviceExt;

/// Maximum number of point lights supported per draw call.
pub const MAX_POINT_LIGHTS: usize = 64;

/// Maximum number of spot lights supported per draw call.
pub const MAX_SPOT_LIGHTS: usize = 32;

/// A directional light (e.g. the sun).
///
/// # Examples
///
/// ```
/// use render::lights::DirectionalLight;
///
/// let light = DirectionalLight::default();
/// assert_eq!(light.intensity, 1.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    /// Normalized direction the light shines towards.
    pub direction: Vec3,
    /// Light color in linear space.
    pub color: Color,
    /// Brightness multiplier.
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

impl fmt::Display for DirectionalLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DirectionalLight(dir: [{:.2}, {:.2}, {:.2}], intensity: {:.2})",
            self.direction.x, self.direction.y, self.direction.z, self.intensity,
        )
    }
}

/// A point light with attenuation.
///
/// # Examples
///
/// ```
/// use render::lights::PointLight;
/// use glam::Vec3;
///
/// let light = PointLight {
///     position: Vec3::new(0.0, 5.0, 0.0),
///     radius: 20.0,
///     ..PointLight::default()
/// };
/// assert_eq!(light.radius, 20.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    /// World-space position of the light.
    pub position: Vec3,
    /// Light color in linear space.
    pub color: Color,
    /// Brightness multiplier.
    pub intensity: f32,
    /// Maximum influence radius.
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

impl fmt::Display for PointLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PointLight(pos: [{:.2}, {:.2}, {:.2}], intensity: {:.2}, radius: {:.1})",
            self.position.x, self.position.y, self.position.z, self.intensity, self.radius,
        )
    }
}

/// A spot light with cone angles.
///
/// # Examples
///
/// ```
/// use render::lights::SpotLight;
/// use glam::Vec3;
///
/// let light = SpotLight {
///     position: Vec3::new(0.0, 10.0, 0.0),
///     direction: Vec3::NEG_Y,
///     inner_cutoff: 0.2,
///     outer_cutoff: 0.4,
///     ..SpotLight::default()
/// };
/// assert!(light.inner_cutoff < light.outer_cutoff);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SpotLight {
    /// World-space position of the light.
    pub position: Vec3,
    /// Direction the spotlight is aimed.
    pub direction: Vec3,
    /// Light color in linear space.
    pub color: Color,
    /// Brightness multiplier.
    pub intensity: f32,
    /// Maximum influence radius.
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

impl fmt::Display for SpotLight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpotLight(pos: [{:.2}, {:.2}, {:.2}], intensity: {:.2}, cutoff: {:.2}..{:.2})",
            self.position.x, self.position.y, self.position.z,
            self.intensity, self.inner_cutoff, self.outer_cutoff,
        )
    }
}

/// Collection of lights in a scene, ready for GPU upload.
///
/// # Examples
///
/// ```
/// use render::lights::{LightSet, PointLight};
///
/// let mut lights = LightSet::default();
/// lights.point_lights.push(PointLight::default());
/// let uniform = lights.to_uniform();
/// assert_eq!(uniform.num_point_lights, 1);
/// ```
#[derive(Debug, Clone)]
pub struct LightSet {
    /// The scene's primary directional light.
    pub directional: DirectionalLight,
    /// All active point lights.
    pub point_lights: Vec<PointLight>,
    /// All active spot lights.
    pub spot_lights: Vec<SpotLight>,
    /// Ambient light color.
    pub ambient_color: Color,
    /// Ambient light intensity multiplier.
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

impl fmt::Display for LightSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LightSet({}, {} point, {} spot, ambient: {:.2})",
            self.directional,
            self.point_lights.len(),
            self.spot_lights.len(),
            self.ambient_intensity,
        )
    }
}

impl LightSet {
    /// Pack into a GPU-ready uniform struct.
    ///
    /// Point and spot light counts are clamped to [`MAX_POINT_LIGHTS`] and
    /// [`MAX_SPOT_LIGHTS`] respectively; excess lights are silently dropped.
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
///
/// The struct layout matches the WGSL `LightUniform` struct expected by the PBR
/// shader. The first three vec4s hold directional light data and ambient color,
/// followed by counts and then the point/spot light arrays.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct LightUniform {
    /// Directional light direction (xyz) with intensity in w.
    pub dir_direction: [f32; 4],
    /// Directional light color (rgb) with intensity in w.
    pub dir_color: [f32; 4],
    /// Ambient light color (rgba, pre-multiplied by intensity).
    pub ambient: [f32; 4],
    /// Number of active point lights.
    pub num_point_lights: u32,
    /// Number of active spot lights.
    pub num_spot_lights: u32,
    /// Padding for std140 alignment.
    pub _pad: [u32; 2],
    /// Point light positions (xyz) with radius in w.
    pub point_positions: [[f32; 4]; MAX_POINT_LIGHTS],
    /// Point light colors (rgb) with intensity in w.
    pub point_colors: [[f32; 4]; MAX_POINT_LIGHTS],
    /// Spot light positions (xyz) with inner cutoff in w.
    pub spot_pos_inner: [[f32; 4]; MAX_SPOT_LIGHTS],
    /// Spot light directions (xyz) with outer cutoff in w.
    pub spot_dir_outer: [[f32; 4]; MAX_SPOT_LIGHTS],
    /// Spot light colors (rgb) with intensity in w.
    pub spot_colors: [[f32; 4]; MAX_SPOT_LIGHTS],
}

/// A GPU buffer holding light uniform data.
pub struct LightsBuffer {
    /// The GPU uniform buffer.
    pub buffer: wgpu::Buffer,
    /// Layout for binding the lights buffer in shaders.
    pub bind_group_layout: wgpu::BindGroupLayout,
    /// Bind group referencing the lights buffer.
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
    #[inline]
    pub fn update(&self, queue: &wgpu::Queue, lights: &LightSet) {
        let uniform = lights.to_uniform();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_light_set_has_zero_point_lights() {
        let lights = LightSet::default();
        let uniform = lights.to_uniform();
        assert_eq!(uniform.num_point_lights, 0);
        assert_eq!(uniform.num_spot_lights, 0);
    }

    #[test]
    fn point_lights_clamped_to_max() {
        let mut lights = LightSet::default();
        for _ in 0..100 {
            lights.point_lights.push(PointLight::default());
        }
        let uniform = lights.to_uniform();
        assert_eq!(uniform.num_point_lights, MAX_POINT_LIGHTS as u32);
    }

    #[test]
    fn spot_lights_clamped_to_max() {
        let mut lights = LightSet::default();
        for _ in 0..50 {
            lights.spot_lights.push(SpotLight::default());
        }
        let uniform = lights.to_uniform();
        assert_eq!(uniform.num_spot_lights, MAX_SPOT_LIGHTS as u32);
    }

    #[test]
    fn directional_light_direction_is_normalized() {
        let light = DirectionalLight::default();
        assert!((light.direction.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn uniform_ambient_values() {
        let lights = LightSet::default();
        let uniform = lights.to_uniform();
        // Default ambient: WHITE * 0.15
        assert!((uniform.ambient[0] - 0.15).abs() < 1e-5);
    }

    #[test]
    fn point_light_packing() {
        let mut lights = LightSet::default();
        lights.point_lights.push(PointLight {
            position: Vec3::new(1.0, 2.0, 3.0),
            color: Color::WHITE,
            intensity: 2.0,
            radius: 15.0,
        });
        let uniform = lights.to_uniform();
        assert_eq!(uniform.num_point_lights, 1);
        assert_eq!(uniform.point_positions[0][0], 1.0);
        assert_eq!(uniform.point_positions[0][1], 2.0);
        assert_eq!(uniform.point_positions[0][2], 3.0);
        assert_eq!(uniform.point_positions[0][3], 15.0); // radius
        assert!((uniform.point_colors[0][0] - 2.0).abs() < 1e-5); // intensity * color
    }

    #[test]
    fn spot_light_packing() {
        let mut lights = LightSet::default();
        lights.spot_lights.push(SpotLight {
            position: Vec3::new(0.0, 5.0, 0.0),
            direction: Vec3::NEG_Y,
            inner_cutoff: 0.3,
            outer_cutoff: 0.5,
            ..SpotLight::default()
        });
        let uniform = lights.to_uniform();
        assert_eq!(uniform.num_spot_lights, 1);
        // inner cutoff stored as cos(angle) in w component of spot_pos_inner
        assert!((uniform.spot_pos_inner[0][3] - 0.3_f32.cos()).abs() < 1e-5);
        // outer cutoff stored as cos(angle) in w component of spot_dir_outer
        assert!((uniform.spot_dir_outer[0][3] - 0.5_f32.cos()).abs() < 1e-5);
    }

    #[test]
    fn display_impls() {
        let _ = format!("{}", DirectionalLight::default());
        let _ = format!("{}", PointLight::default());
        let _ = format!("{}", SpotLight::default());
        let _ = format!("{}", LightSet::default());
    }

    #[test]
    fn light_uniform_size() {
        // Verify the uniform struct has expected size
        let size = std::mem::size_of::<LightUniform>();
        // dir_direction(16) + dir_color(16) + ambient(16) + counts(16) +
        // point_positions(64*16) + point_colors(64*16) +
        // spot_pos_inner(32*16) + spot_dir_outer(32*16) + spot_colors(32*16)
        let expected = 16 + 16 + 16 + 16 + (64 * 16) + (64 * 16) + (32 * 16) + (32 * 16) + (32 * 16);
        assert_eq!(size, expected);
    }
}
