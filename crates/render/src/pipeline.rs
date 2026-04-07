//! Render pipeline creation and inline WGSL shaders.
//!
//! Contains all WGSL shader source strings (PBR, wireframe, unlit, normals,
//! depth, grid) and the [`PipelineCache`] that lazily creates and caches
//! `wgpu::RenderPipeline` instances keyed by [`RenderStyle`].

use std::collections::HashMap;

use crate::render_style::RenderStyle;
use crate::vertex::Vertex;

// ─────────────────────────────────────────────────────────────────────
// WGSL shaders
// ─────────────────────────────────────────────────────────────────────

/// PBR lit shader -- single directional light, Cook-Torrance GGX specular.
pub const SHADER_PBR: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view:      mat4x4<f32>,
    proj:      mat4x4<f32>,
    eye_pos:   vec3<f32>,
    _pad:      f32,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct LightUniform {
    dir_direction: vec4<f32>,
    dir_color:     vec4<f32>,
    ambient:       vec4<f32>,
    num_point:     u32,
    num_spot:      u32,
    _pad0:         u32,
    _pad1:         u32,
};
@group(1) @binding(0) var<uniform> lights: LightUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) color:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos:  vec3<f32>,
    @location(1) world_norm: vec3<f32>,
    @location(2) uv:         vec2<f32>,
    @location(3) color:      vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos   = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos  = in.position;
    out.world_norm = in.normal;
    out.uv         = in.uv;
    out.color      = in.color;
    return out;
}

const PI: f32 = 3.14159265359;

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a  = roughness * roughness;
    let a2 = a * a;
    let d  = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * d * d + 0.0001);
}

fn geometry_schlick(n_dot_v: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k + 0.0001);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let albedo    = in.color.rgb;
    let metallic  = 0.0;
    let roughness = 0.5;

    let n = normalize(in.world_norm);
    let v = normalize(camera.eye_pos - in.world_pos);

    // Directional light
    let l = normalize(-lights.dir_direction.xyz);
    let h = normalize(v + l);

    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_v = max(dot(n, v), 0.001);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, v), 0.0);

    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    let d = distribution_ggx(n_dot_h, roughness);
    let g = geometry_schlick(n_dot_l, roughness) * geometry_schlick(n_dot_v, roughness);
    let f = fresnel_schlick(h_dot_v, f0);

    let spec = (d * g * f) / (4.0 * n_dot_v * n_dot_l + 0.0001);
    let kd   = (vec3<f32>(1.0) - f) * (1.0 - metallic);

    let radiance = lights.dir_color.rgb;
    let lo = (kd * albedo / PI + spec) * radiance * n_dot_l;

    let ambient = lights.ambient.rgb * albedo;
    let color   = ambient + lo;

    // Simple Reinhard tone mapping
    let mapped = color / (color + vec3<f32>(1.0));

    return vec4<f32>(mapped, in.color.a);
}
"#;

/// Wireframe shader -- flat white lines.
pub const SHADER_WIREFRAME: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view:      mat4x4<f32>,
    proj:      mat4x4<f32>,
    eye_pos:   vec3<f32>,
    _pad:      f32,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) color:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(in.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.8, 0.8, 0.8, 1.0);
}
"#;

/// Unlit shader -- flat vertex color, no lighting.
pub const SHADER_UNLIT: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view:      mat4x4<f32>,
    proj:      mat4x4<f32>,
    eye_pos:   vec3<f32>,
    _pad:      f32,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) color:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color    = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

/// Normals visualization shader -- world-space normals mapped to RGB.
pub const SHADER_NORMALS: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view:      mat4x4<f32>,
    proj:      mat4x4<f32>,
    eye_pos:   vec3<f32>,
    _pad:      f32,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) color:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.normal   = in.normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let n = normalize(in.normal) * 0.5 + 0.5;
    return vec4<f32>(n, 1.0);
}
"#;

/// Depth visualization shader -- near = white, far = black.
pub const SHADER_DEPTH: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view:      mat4x4<f32>,
    proj:      mat4x4<f32>,
    eye_pos:   vec3<f32>,
    _pad:      f32,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
    @location(3) color:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) view_depth: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let view_pos = camera.view * vec4<f32>(in.position, 1.0);
    out.clip_pos   = camera.proj * view_pos;
    out.view_depth = -view_pos.z;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let near = 0.1;
    let far  = 100.0;
    let linear = clamp((in.view_depth - near) / (far - near), 0.0, 1.0);
    let grey   = 1.0 - linear;
    return vec4<f32>(grey, grey, grey, 1.0);
}
"#;

/// Grid shader -- infinite ground plane grid with distance-based fade.
pub const SHADER_GRID: &str = r#"
struct CameraUniform {
    view_proj: mat4x4<f32>,
    view:      mat4x4<f32>,
    proj:      mat4x4<f32>,
    eye_pos:   vec3<f32>,
    _pad:      f32,
};
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    // Full-screen quad via 6 vertices covering a large ground plane
    let positions = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
    );
    let pos = positions[idx];

    var out: VertexOutput;
    // Place a large ground-plane quad centered around the camera XZ
    let extent = 100.0;
    let world = vec3<f32>(
        pos.x * extent + camera.eye_pos.x,
        0.0,
        pos.y * extent + camera.eye_pos.z,
    );
    out.clip_pos  = camera.view_proj * vec4<f32>(world, 1.0);
    out.world_pos = world;
    return out;
}

fn grid_line(coord: vec2<f32>, scale: f32) -> f32 {
    let d = fract(coord / scale + 0.5) - 0.5;
    let g = abs(d) / (fwidth(d) + 0.0001);
    return 1.0 - min(min(g.x, g.y), 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_xz = in.world_pos.xz;
    let dist = length(in.world_pos - camera.eye_pos);

    // Fine and coarse grid lines
    let fine   = grid_line(world_xz, 1.0);
    let coarse = grid_line(world_xz, 10.0);
    let line   = max(fine * 0.3, coarse * 0.6);

    // Fade with distance
    let fade = 1.0 - smoothstep(20.0, 80.0, dist);
    let alpha = line * fade;

    if alpha < 0.01 {
        discard;
    }

    // Axis coloring: red for X axis, blue for Z axis
    var color = vec3<f32>(0.4, 0.4, 0.4);
    if abs(world_xz.y) < 0.1 {
        color = vec3<f32>(0.8, 0.2, 0.2);
    }
    if abs(world_xz.x) < 0.1 {
        color = vec3<f32>(0.2, 0.2, 0.8);
    }

    return vec4<f32>(color, alpha);
}
"#;

// ─────────────────────────────────────────────────────────────────────
// Pipeline Cache
// ─────────────────────────────────────────────────────────────────────

/// Caches render pipelines keyed by `RenderStyle`.
pub struct PipelineCache {
    pipelines: HashMap<RenderStyle, wgpu::RenderPipeline>,
    grid_pipeline: Option<wgpu::RenderPipeline>,
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineCache {
    /// Creates a new, empty pipeline cache.
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            grid_pipeline: None,
        }
    }

    /// Get or create the pipeline for a given render style.
    pub fn get_or_create(
        &mut self,
        style: RenderStyle,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_bgl: &wgpu::BindGroupLayout,
        lights_bgl: &wgpu::BindGroupLayout,
    ) -> &wgpu::RenderPipeline {
        self.pipelines.entry(style).or_insert_with(|| {
            create_mesh_pipeline(device, surface_format, style, camera_bgl, lights_bgl)
        })
    }

    /// Get or create the grid pipeline.
    pub fn get_or_create_grid(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        camera_bgl: &wgpu::BindGroupLayout,
    ) -> &wgpu::RenderPipeline {
        self.grid_pipeline.get_or_insert_with(|| {
            create_grid_pipeline(device, surface_format, camera_bgl)
        })
    }

    /// Invalidate all cached pipelines (e.g. on surface format change).
    pub fn clear(&mut self) {
        self.pipelines.clear();
        self.grid_pipeline = None;
    }
}

/// Returns the WGSL shader source string for the given render style.
fn shader_source_for_style(style: RenderStyle) -> &'static str {
    match style {
        RenderStyle::Pbr => SHADER_PBR,
        RenderStyle::Wireframe => SHADER_WIREFRAME,
        RenderStyle::Unlit => SHADER_UNLIT,
        RenderStyle::Normals => SHADER_NORMALS,
        RenderStyle::Depth => SHADER_DEPTH,
    }
}

/// Create a render pipeline for mesh drawing with the given style.
fn create_mesh_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    style: RenderStyle,
    camera_bgl: &wgpu::BindGroupLayout,
    lights_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let source = shader_source_for_style(style);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mesh_shader"),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });

    // PBR needs both camera + lights bind groups; others only camera.
    let bind_group_layouts: Vec<&wgpu::BindGroupLayout> = if style.needs_lighting() {
        vec![camera_bgl, lights_bgl]
    } else {
        vec![camera_bgl]
    };

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mesh_pipeline_layout"),
        bind_group_layouts: &bind_group_layouts,
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mesh_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Vertex::buffer_layout()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: crate::gpu::GpuContext::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

/// Create a render pipeline for the infinite ground-plane grid overlay.
fn create_grid_pipeline(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    camera_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("grid_shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER_GRID.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("grid_pipeline_layout"),
        bind_group_layouts: &[camera_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("grid_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: crate::gpu::GpuContext::DEPTH_FORMAT,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}
