//! GPU device and surface management via wgpu.
//!
//! [`GpuContext`] owns the wgpu instance, adapter, device, and queue.
//! [`SurfaceState`] wraps a window surface and its configuration.

use anyhow::Result;
use tracing::info;

/// Holds the wgpu instance, adapter, device, and queue.
///
/// Created via [`GpuContext::new`], which requests a high-performance GPU adapter.
pub struct GpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    /// Create a new GPU context (headless / off-screen).
    ///
    /// Requests a high-performance adapter with all available backends.
    /// Returns an error if no suitable GPU adapter is found or if device
    /// creation fails.
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No suitable GPU adapter found. Ensure a GPU with Vulkan, Metal, \
                     or DX12 support is available."
                )
            })?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("forge-render-device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    ..Default::default()
                },
                None,
            )
            .await?;

        info!(
            adapter = ?adapter.get_info().name,
            backend = ?adapter.get_info().backend,
            "GPU context created"
        );

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    /// Depth texture format used throughout the pipeline.
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
}

/// Manages a wgpu surface and its configuration for window rendering.
pub struct SurfaceState {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl SurfaceState {
    /// Create a new surface state from an existing surface.
    ///
    /// Automatically selects an sRGB format when available, falling back to
    /// the first supported format. Dimensions are clamped to at least 1x1.
    pub fn new(
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> Self {
        let caps = surface.get_capabilities(adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.max(1),
            height: height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(device, &config);

        Self { surface, config }
    }

    /// Resize the surface. No-op if either dimension is zero.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(device, &self.config);
    }

    /// Returns the surface texture format.
    #[inline]
    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Returns `(width, height)` of the surface.
    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
}
