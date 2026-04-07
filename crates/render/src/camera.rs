//! Camera and projection types, plus GPU uniform buffer.
//!
//! [`Camera`] produces view and projection matrices. [`CameraBuffer`] manages
//! the GPU-side uniform buffer and bind group for the camera data.
//!
//! # Examples
//!
//! ```no_run
//! use render::camera::{Camera, Projection};
//!
//! let mut cam = Camera::default();
//! cam.set_aspect(1920.0, 1080.0);
//! let vp = cam.view_projection();
//! ```

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

/// Projection mode.
///
/// # Examples
///
/// ```
/// use render::camera::Projection;
///
/// let proj = Projection::default();
/// assert!(matches!(proj, Projection::Perspective { .. }));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Projection {
    Perspective {
        fov_y_radians: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        height: f32,
        near: f32,
        far: f32,
    },
}

impl Default for Projection {
    fn default() -> Self {
        Self::Perspective {
            fov_y_radians: std::f32::consts::FRAC_PI_4,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl std::fmt::Display for Projection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Perspective {
                fov_y_radians,
                near,
                far,
            } => write!(
                f,
                "Perspective(fov: {:.1} deg, near: {near}, far: {far})",
                fov_y_radians.to_degrees()
            ),
            Self::Orthographic { height, near, far } => {
                write!(f, "Orthographic(height: {height}, near: {near}, far: {far})")
            }
        }
    }
}

/// A 3-D camera that produces view and projection matrices.
///
/// # Examples
///
/// ```
/// use render::camera::Camera;
///
/// let cam = Camera::default();
/// let view = cam.view_matrix();
/// let proj = cam.projection_matrix();
/// let vp = cam.view_projection();
/// ```
#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub projection: Projection,
    pub aspect: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: Vec3::new(0.0, 5.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: Projection::default(),
            aspect: 16.0 / 9.0,
        }
    }
}

impl std::fmt::Display for Camera {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Camera(eye: [{:.1}, {:.1}, {:.1}], target: [{:.1}, {:.1}, {:.1}], aspect: {:.2}, {})",
            self.eye.x, self.eye.y, self.eye.z,
            self.target.x, self.target.y, self.target.z,
            self.aspect,
            self.projection,
        )
    }
}

impl Camera {
    /// Compute the view matrix (world -> camera).
    ///
    /// # Examples
    ///
    /// ```
    /// use render::camera::Camera;
    /// use glam::Mat4;
    ///
    /// let cam = Camera::default();
    /// let view = cam.view_matrix();
    /// // View matrix should be invertible
    /// let inv = view.inverse();
    /// let roundtrip = view * inv;
    /// assert!((roundtrip.col(0).x - 1.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye, self.target, self.up)
    }

    /// Compute the projection matrix.
    #[inline]
    pub fn projection_matrix(&self) -> Mat4 {
        match self.projection {
            Projection::Perspective {
                fov_y_radians,
                near,
                far,
            } => Mat4::perspective_rh(fov_y_radians, self.aspect, near, far),
            Projection::Orthographic { height, near, far } => {
                let half_h = height * 0.5;
                let half_w = half_h * self.aspect;
                Mat4::orthographic_rh(-half_w, half_w, -half_h, half_h, near, far)
            }
        }
    }

    /// Combined view-projection matrix.
    #[inline]
    pub fn view_projection(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Build the GPU-ready uniform data.
    #[inline]
    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.view_projection().to_cols_array_2d(),
            view: self.view_matrix().to_cols_array_2d(),
            proj: self.projection_matrix().to_cols_array_2d(),
            eye_pos: self.eye.to_array(),
            _pad: 0.0,
        }
    }

    /// Update the aspect ratio (e.g. on window resize).
    ///
    /// If `height` is zero or negative, the aspect ratio is left unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// use render::camera::Camera;
    ///
    /// let mut cam = Camera::default();
    /// cam.set_aspect(1920.0, 1080.0);
    /// assert!((cam.aspect - 1920.0 / 1080.0).abs() < 1e-5);
    ///
    /// // Zero height is a no-op
    /// cam.set_aspect(800.0, 0.0);
    /// assert!((cam.aspect - 1920.0 / 1080.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn set_aspect(&mut self, width: f32, height: f32) {
        if height > 0.0 {
            self.aspect = width / height;
        }
    }

    /// Returns the forward direction the camera is looking at (normalized).
    ///
    /// # Examples
    ///
    /// ```
    /// use render::camera::Camera;
    /// use glam::Vec3;
    ///
    /// let cam = Camera::default();
    /// let fwd = cam.forward();
    /// assert!((fwd.length() - 1.0).abs() < 1e-5);
    /// ```
    #[inline]
    pub fn forward(&self) -> Vec3 {
        (self.target - self.eye).normalize_or_zero()
    }
}

/// GPU-side camera uniform (std140 layout).
///
/// Fields are laid out to match the WGSL `CameraUniform` struct used in all
/// shaders: `view_proj`, `view`, `proj` as `mat4x4<f32>`, then `eye_pos` as
/// `vec3<f32>` with one float of padding.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub eye_pos: [f32; 3],
    pub _pad: f32,
}

/// A GPU buffer holding camera uniform data, ready to bind.
pub struct CameraBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl CameraBuffer {
    /// Create a new camera buffer with initial data.
    pub fn new(device: &wgpu::Device, camera: &Camera) -> Self {
        let uniform = camera.to_uniform();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_uniform"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
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

    /// Upload new camera data to the GPU.
    #[inline]
    pub fn update(&self, queue: &wgpu::Queue, camera: &Camera) {
        let uniform = camera.to_uniform();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_camera_produces_valid_matrices() {
        let cam = Camera::default();
        let vp = cam.view_projection();
        // VP matrix should be finite
        for col in 0..4 {
            for row in 0..4 {
                assert!(vp.col(col)[row].is_finite());
            }
        }
    }

    #[test]
    fn set_aspect_zero_height_is_noop() {
        let mut cam = Camera::default();
        let orig = cam.aspect;
        cam.set_aspect(800.0, 0.0);
        assert_eq!(cam.aspect, orig);
    }

    #[test]
    fn set_aspect_normal() {
        let mut cam = Camera::default();
        cam.set_aspect(1920.0, 1080.0);
        assert!((cam.aspect - 1920.0 / 1080.0).abs() < 1e-5);
    }

    #[test]
    fn orthographic_projection() {
        let cam = Camera {
            projection: Projection::Orthographic {
                height: 10.0,
                near: 0.1,
                far: 100.0,
            },
            ..Camera::default()
        };
        let proj = cam.projection_matrix();
        for col in 0..4 {
            for row in 0..4 {
                assert!(proj.col(col)[row].is_finite());
            }
        }
    }

    #[test]
    fn camera_forward() {
        let cam = Camera::default();
        let fwd = cam.forward();
        assert!((fwd.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn uniform_layout_size() {
        // Verify the uniform is the expected size for std140
        let size = std::mem::size_of::<CameraUniform>();
        // 3 mat4x4 (3 * 64) + vec3 + pad (16) = 208 bytes
        assert_eq!(size, 208);
    }

    #[test]
    fn projection_display() {
        let p = Projection::default();
        let s = format!("{p}");
        assert!(s.contains("Perspective"));
    }

    #[test]
    fn camera_display() {
        let cam = Camera::default();
        let s = format!("{cam}");
        assert!(s.contains("Camera"));
    }
}
