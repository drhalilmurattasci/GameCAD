//! High-level renderer orchestrating a full frame.
//!
//! [`Renderer`] owns the camera and light uniform buffers, the depth texture,
//! the pipeline cache, and the current render style. Call [`Renderer::render_frame`]
//! each frame to produce the final image.

use crate::camera::{Camera, CameraBuffer};
use crate::gpu::GpuContext;
use crate::lights::{LightSet, LightsBuffer};
use crate::pipeline::PipelineCache;
use crate::render_style::RenderStyle;
use crate::texture::Texture;
use crate::vertex::GpuMesh;

/// Orchestrates a full rendering frame: clear, depth, main pass, grid, present.
pub struct Renderer {
    pub camera_buffer: CameraBuffer,
    pub lights_buffer: LightsBuffer,
    pub pipeline_cache: PipelineCache,
    pub depth_texture: Texture,
    pub render_style: RenderStyle,
    pub show_grid: bool,
    pub clear_color: wgpu::Color,
    surface_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
}

impl Renderer {
    /// Create a new renderer for the given surface format and dimensions.
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let camera = Camera::default();
        let lights = LightSet::default();

        let camera_buffer = CameraBuffer::new(device, &camera);
        let lights_buffer = LightsBuffer::new(device, &lights);
        let depth_texture =
            Texture::create_depth_texture(device, width, height, Some("depth_texture"));

        Self {
            camera_buffer,
            lights_buffer,
            pipeline_cache: PipelineCache::new(),
            depth_texture,
            render_style: RenderStyle::default(),
            show_grid: true,
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
            surface_format,
            width,
            height,
        }
    }

    /// Current surface format.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// Current dimensions.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Resize internal resources (depth texture) to match new dimensions.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.width = width;
        self.height = height;
        self.depth_texture =
            Texture::create_depth_texture(device, width, height, Some("depth_texture"));
    }

    /// Update camera uniform data on the GPU.
    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &Camera) {
        self.camera_buffer.update(queue, camera);
    }

    /// Update light uniform data on the GPU.
    pub fn update_lights(&self, queue: &wgpu::Queue, lights: &LightSet) {
        self.lights_buffer.update(queue, lights);
    }

    /// Render a full frame to the given color target view.
    ///
    /// `meshes` is a slice of GPU meshes to draw in the main pass.
    pub fn render_frame(
        &mut self,
        gpu: &GpuContext,
        target_view: &wgpu::TextureView,
        meshes: &[&GpuMesh],
    ) {
        let device = &gpu.device;
        let queue = &gpu.queue;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        // ── Main pass: clear + draw meshes ──────────────────────────
        {
            let pipeline = self.pipeline_cache.get_or_create(
                self.render_style,
                device,
                self.surface_format,
                &self.camera_buffer.bind_group_layout,
                &self.lights_buffer.bind_group_layout,
            );

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);
            if self.render_style.needs_lighting() {
                pass.set_bind_group(1, &self.lights_buffer.bind_group, &[]);
            }

            for mesh in meshes {
                pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                pass.set_index_buffer(mesh.index_buffer.slice(..), mesh.index_format);
                pass.draw_indexed(0..mesh.index_count, 0, 0..1);
            }
        }

        // ── Grid pass (transparent, after main geometry) ────────────
        if self.show_grid {
            let grid_pipeline = self.pipeline_cache.get_or_create_grid(
                device,
                self.surface_format,
                &self.camera_buffer.bind_group_layout,
            );

            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("grid_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(grid_pipeline);
            pass.set_bind_group(0, &self.camera_buffer.bind_group, &[]);
            pass.draw(0..6, 0..1);
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
