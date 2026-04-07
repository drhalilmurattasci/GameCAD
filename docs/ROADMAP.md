# Forge Editor — Modular Rust CAD/Game Editor

> A Crystalline-themed, MVVM-architected modular editor combining the precision of Rhino 3D, the parametric power of Fusion 360, and the game-ready workflow of Unreal Engine. Built in Rust with wgpu + egui.

---

## Part 1: Design Brainstorm

### 1.1 Visual Identity — Crystalline Theme

The visual identity draws from Baby Audio's "Crystalline" reverb plugin: a dark, atmospheric interface punctuated by vivid cyan-green and electric blue accents. The aesthetic is "future-retro" -- clean geometry, frosted glass panels, and morphing interactive elements that feel alive without being distracting.

#### Color Palette

| Token                  | Hex       | Usage                                      |
|------------------------|-----------|---------------------------------------------|
| `base-900`             | `#131316` | Deepest background, viewport surround       |
| `base-800`             | `#1a1a1e` | Docked panel backgrounds                    |
| `base-700`             | `#1f1f21` | Primary panel background (charcoal)         |
| `base-600`             | `#262629` | Secondary panel / raised surfaces           |
| `base-500`             | `#2e2e32` | Card / inset surfaces                       |
| `base-400`             | `#3a3a3f` | Hover backgrounds, subtle dividers          |
| `base-300`             | `#4a4a50` | Disabled element backgrounds                |
| `surface-glass`        | `#1f1f21cc` | Frosted glass panel (80% opacity)         |
| `surface-glass-hover`  | `#262629e6` | Frosted glass hover (90% opacity)         |
| `accent-primary`       | `#4eff93` | Cyan-green -- primary interactive accent    |
| `accent-primary-dim`   | `#4eff9366` | Accent at 40% -- subtle highlights        |
| `accent-primary-glow`  | `#4eff9333` | Accent at 20% -- selection glow           |
| `accent-secondary`     | `#3e55ff` | Electric blue -- secondary accent           |
| `accent-secondary-dim` | `#3e55ff66` | Blue at 40% -- secondary highlights       |
| `accent-tertiary`      | `#e94560` | Red-pink -- warnings, X-axis               |
| `accent-warm`          | `#ff9f43` | Warm orange -- notifications, caution       |
| `text-primary`         | `#e7e7ea` | Primary text (off-white)                    |
| `text-secondary`       | `#9b9ba1` | Secondary text (neutral gray)               |
| `text-tertiary`        | `#6b6b72` | Tertiary text, placeholders                 |
| `text-disabled`        | `#4a4a50` | Disabled text                               |
| `text-on-accent`       | `#0a0a14` | Text rendered on accent backgrounds         |
| `border-subtle`        | `#2e2e3280` | Panel borders (50% opacity)               |
| `border-normal`        | `#3a3a3f`   | Normal borders                            |
| `border-focus`         | `#4eff93`   | Focus ring color                          |
| `border-error`         | `#e94560`   | Error state borders                       |
| `state-success`        | `#4eff93` | Success indicators                          |
| `state-warning`        | `#ff9f43` | Warning indicators                          |
| `state-error`          | `#e94560` | Error indicators                            |
| `state-info`           | `#3e55ff` | Info indicators                             |
| `viewport-top`         | `#0a0a14` | Viewport gradient top                       |
| `viewport-bottom`      | `#1a1a2e` | Viewport gradient bottom                    |
| `gizmo-x`             | `#e94560` | X-axis (red-pink)                           |
| `gizmo-y`             | `#4eff93` | Y-axis (cyan-green / Crystalline accent)    |
| `gizmo-z`             | `#3e55ff` | Z-axis (electric blue)                      |
| `gizmo-screen`        | `#e7e7ea` | Screen-space handle                         |
| `selection-outline`    | `#4eff93` | Selected object outline                     |
| `selection-fill`       | `#4eff9333` | Selection fill (20% alpha glow)           |

#### Dark Mode / Light Mode

Dark mode is the default and primary design target. Light mode is available as a toggle and inverts the luminance scale while preserving accent hues:

| Dark Token   | Light Equivalent | Light Hex   |
|--------------|------------------|-------------|
| `base-900`   | `light-50`       | `#f5f5f7`   |
| `base-800`   | `light-100`      | `#ebebef`   |
| `base-700`   | `light-200`      | `#e0e0e5`   |
| `base-600`   | `light-300`      | `#d0d0d8`   |
| `text-primary` | `light-text`   | `#1a1a1e`   |
| `text-secondary` | `light-muted` | `#5a5a62` |

Accent colors remain unchanged in light mode to preserve brand identity.

#### Panel Styling: Frosted Glass Effect

Panels use a semi-transparent background with a backdrop blur to create the frosted glass aesthetic:

```css
/* Conceptual -- implemented via egui custom painting */
panel {
  background: rgba(31, 31, 33, 0.80);     /* surface-glass */
  backdrop-filter: blur(24px);
  border: 1px solid rgba(46, 46, 50, 0.50); /* border-subtle */
  border-radius: 8px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.40);
}
```

In egui, this is achieved by rendering a blurred copy of the region behind the panel, then overlaying the tinted semi-transparent surface. The `render` crate provides a dedicated blur pass for this.

#### Interactive Elements: Morphing Buttons

Buttons and sliders feature gradient morphing based on interaction state:

- **Idle**: Flat `base-500` fill, `text-secondary` label
- **Hover**: Linear gradient from `accent-primary-dim` (left) to `accent-secondary-dim` (right), label shifts to `text-primary`
- **Active/Pressed**: Solid `accent-primary` fill, `text-on-accent` label, scale 0.97x
- **Value-linked morphing**: Slider knobs shift their gradient hue based on the current value (low = blue, mid = green, high = warm orange), mimicking Crystalline's reverb decay knob

#### Typography

| Role            | Font             | Weight    | Size    |
|-----------------|------------------|-----------|---------|
| Panel titles    | Inter            | SemiBold  | 13px    |
| Labels          | Inter            | Regular   | 12px    |
| Body text       | Inter            | Regular   | 13px    |
| Small captions  | Inter            | Regular   | 10px    |
| Code / values   | JetBrains Mono   | Regular   | 12px    |
| Command line    | JetBrains Mono   | Regular   | 13px    |
| Node labels     | Inter            | Medium    | 11px    |
| Headings        | Inter            | Bold      | 15px    |

#### Icon Style

- Format: SVG line icons
- Stroke width: 1.5px
- Default color: `text-secondary` (`#9b9ba1`)
- Hover color: `text-primary` (`#e7e7ea`)
- Active color: `accent-primary` (`#4eff93`)
- Size: 16x16 (toolbar), 20x20 (panel headers), 12x12 (inline)
- Corners: rounded line caps and joins
- No fills unless explicitly needed for toggle-on states

#### Viewport Background

The 3D viewport uses a vertical gradient to ground the scene:

- Top: `#0a0a14` (near-black with blue undertone)
- Bottom: `#1a1a2e` (dark navy-purple)

Grid lines render at `#2e2e3240` (25% opacity), with major grid lines at `#2e2e3280` (50% opacity).

#### Gizmo Colors

| Axis    | Color     | Hex       |
|---------|-----------|-----------|
| X       | Red-Pink  | `#e94560` |
| Y       | Cyan-Green| `#4eff93` |
| Z       | Electric Blue | `#3e55ff` |
| Screen  | Off-White | `#e7e7ea` |
| Plane XY| Blended   | `#4eff9380` |
| Plane XZ| Blended   | `#e9456080` |
| Plane YZ| Blended   | `#3e55ff80` |

#### Selection Highlight

- Outline: `#4eff93` at 2px (solid)
- Glow: `#4eff93` at 20% alpha, 4px feathered bloom around selection
- Multi-select: additive glow stacking
- Hover preview: `#4eff93` at 10% alpha dashed outline

#### Nested Panel Progressive Darkening

Each level of panel nesting darkens by one step:

| Level | Background   | Token       |
|-------|-------------|-------------|
| 0     | `#1f1f21`   | `base-700`  |
| 1     | `#1a1a1e`   | `base-800`  |
| 2     | `#131316`   | `base-900`  |
| 3     | `#0f0f12`   | (computed)  |

#### Toolbar and Status Bar

- **Toolbar** (top): Frosted glass strip, 36px height, `surface-glass` background, icons spaced 32px apart, subtle bottom border `border-subtle`
- **Status bar** (bottom): Frosted glass strip, 24px height, `surface-glass` background, displays: coordinate readout, snap status, polygon count, FPS, memory usage, current tool/mode name

#### Full Theme File: `crystalline.theme.toml`

```toml
[theme]
name = "Crystalline"
version = "1.0.0"
author = "Forge Team"
description = "Dark future-retro theme inspired by Baby Audio Crystalline"
default_mode = "dark"

[dark]
# ── Backgrounds ──
base_900        = "#131316"
base_800        = "#1a1a1e"
base_700        = "#1f1f21"
base_600        = "#262629"
base_500        = "#2e2e32"
base_400        = "#3a3a3f"
base_300        = "#4a4a50"

# ── Glass Surfaces ──
surface_glass         = "#1f1f21cc"
surface_glass_hover   = "#262629e6"
surface_glass_active  = "#2e2e32f0"
glass_blur_radius     = 24.0
glass_border_radius   = 8.0

# ── Accents ──
accent_primary       = "#4eff93"
accent_primary_dim   = "#4eff9366"
accent_primary_glow  = "#4eff9333"
accent_secondary     = "#3e55ff"
accent_secondary_dim = "#3e55ff66"
accent_tertiary      = "#e94560"
accent_warm          = "#ff9f43"

# ── Text ──
text_primary    = "#e7e7ea"
text_secondary  = "#9b9ba1"
text_tertiary   = "#6b6b72"
text_disabled   = "#4a4a50"
text_on_accent  = "#0a0a14"

# ── Borders ──
border_subtle  = "#2e2e3280"
border_normal  = "#3a3a3f"
border_focus   = "#4eff93"
border_error   = "#e94560"

# ── States ──
state_success  = "#4eff93"
state_warning  = "#ff9f43"
state_error    = "#e94560"
state_info     = "#3e55ff"

# ── Viewport ──
viewport_gradient_top    = "#0a0a14"
viewport_gradient_bottom = "#1a1a2e"
grid_major               = "#2e2e3280"
grid_minor               = "#2e2e3240"

# ── Gizmo ──
gizmo_x       = "#e94560"
gizmo_y       = "#4eff93"
gizmo_z       = "#3e55ff"
gizmo_screen  = "#e7e7ea"
gizmo_plane_xy = "#4eff9380"
gizmo_plane_xz = "#e9456080"
gizmo_plane_yz = "#3e55ff80"

# ── Selection ──
selection_outline     = "#4eff93"
selection_fill        = "#4eff9333"
selection_hover       = "#4eff931a"
selection_outline_px  = 2.0
selection_glow_px     = 4.0

# ── Scrollbar ──
scrollbar_track    = "#1a1a1e"
scrollbar_thumb    = "#3a3a3f"
scrollbar_hover    = "#4a4a50"
scrollbar_width    = 6.0

# ── Shadows ──
shadow_color  = "#00000066"
shadow_offset = [0.0, 4.0]
shadow_blur   = 24.0

[dark.toolbar]
height            = 36.0
background        = "#1f1f21cc"
icon_size         = 16.0
icon_spacing      = 32.0
border_bottom     = "#2e2e3280"

[dark.statusbar]
height            = 24.0
background        = "#1f1f21cc"
font_size         = 10.0
text_color        = "#9b9ba1"

[dark.panel]
header_height     = 28.0
header_background = "#1a1a1e"
content_padding   = 8.0
nesting_darken    = 0.06

[dark.button]
idle_bg           = "#2e2e32"
idle_text         = "#9b9ba1"
hover_gradient    = ["#4eff9366", "#3e55ff66"]
active_bg         = "#4eff93"
active_text       = "#0a0a14"
border_radius     = 6.0
height            = 28.0
padding_h         = 12.0

[dark.input]
background        = "#1a1a1e"
border            = "#2e2e3280"
focus_border      = "#4eff93"
text              = "#e7e7ea"
placeholder       = "#6b6b72"
border_radius     = 4.0
height            = 26.0

[dark.slider]
track_bg          = "#1a1a1e"
track_fill        = "#4eff93"
thumb_size        = 14.0
thumb_gradient    = ["#3e55ff", "#4eff93", "#ff9f43"]

[dark.tabs]
idle_bg           = "transparent"
idle_text         = "#6b6b72"
active_bg         = "#1f1f21"
active_text       = "#e7e7ea"
indicator         = "#4eff93"
indicator_height  = 2.0

[dark.tree]
indent            = 16.0
row_height        = 22.0
hover_bg          = "#2e2e3240"
selected_bg       = "#4eff9320"
connector_color   = "#3a3a3f"

[dark.tooltip]
background        = "#262629f0"
border            = "#3a3a3f"
text              = "#e7e7ea"
border_radius     = 4.0
delay_ms          = 500

[dark.context_menu]
background        = "#262629f0"
border            = "#3a3a3f"
hover_bg          = "#4eff9320"
separator         = "#2e2e3280"
border_radius     = 6.0
shadow_blur       = 16.0

[dark.node_editor]
background        = "#131316"
grid_size         = 20.0
grid_color        = "#1f1f2140"
node_bg           = "#1f1f21e6"
node_border       = "#2e2e32"
node_header       = "#262629"
pin_float         = "#4eff93"
pin_vector        = "#3e55ff"
pin_color         = "#e94560"
pin_bool          = "#ff9f43"
pin_texture       = "#9b9ba1"
wire_thickness    = 2.0
wire_curvature    = 0.5

[light]
base_900        = "#f5f5f7"
base_800        = "#ebebef"
base_700        = "#e0e0e5"
base_600        = "#d0d0d8"
base_500        = "#c0c0c8"
base_400        = "#b0b0b8"
base_300        = "#a0a0a8"
surface_glass   = "#e0e0e5cc"
text_primary    = "#1a1a1e"
text_secondary  = "#5a5a62"
text_tertiary   = "#7a7a82"
accent_primary  = "#4eff93"
accent_secondary = "#3e55ff"
accent_tertiary = "#e94560"
viewport_gradient_top    = "#d0d0d8"
viewport_gradient_bottom = "#e0e0e5"

[fonts]
ui_family         = "Inter"
ui_weight         = 400
ui_size           = 13.0
mono_family       = "JetBrains Mono"
mono_weight       = 400
mono_size         = 12.0
heading_weight    = 700
heading_size      = 15.0
caption_size      = 10.0

[icons]
format            = "svg"
stroke_width      = 1.5
default_color     = "#9b9ba1"
hover_color       = "#e7e7ea"
active_color      = "#4eff93"
size_toolbar      = 16.0
size_panel_header = 20.0
size_inline       = 12.0
line_cap          = "round"
line_join         = "round"

[animation]
hover_duration_ms     = 150
active_duration_ms    = 80
panel_slide_ms        = 200
morphing_easing       = "ease-out-cubic"
glow_pulse_period_ms  = 2000
```

---

### 1.2 Interface Layout — Fusion x Unreal x Rhino

#### Borrowed from Fusion 360

| Feature                 | Adaptation in Forge                                     |
|-------------------------|---------------------------------------------------------|
| Timeline bar            | Bottom strip: parametric history + animation keyframes  |
| Component browser       | Left panel: nested assembly tree with drag-drop reorder |
| Sketch mode             | 2D drawing on construction planes with constraints      |
| Joint / constraint system| Parametric joints stored as timeline operations         |
| Data panel              | Cloud projects sidebar, version history, branching      |
| Design history          | Each operation is a timeline node, editable retroactively|
| Browser bar             | Component + body + sketch hierarchy                     |

#### Borrowed from Unreal Engine

| Feature              | Adaptation in Forge                                      |
|----------------------|----------------------------------------------------------|
| Viewport controls    | Right-click orbit, MMB pan, scroll zoom, WASD fly mode   |
| Content browser      | Bottom panel: thumbnail grid + folder tree for assets    |
| Details panel        | Right panel: property inspector with categories          |
| Outliner             | Top-right: full scene hierarchy, search, filter, type icons |
| Modes panel          | Left toolbar: Select, Translate, Rotate, Scale, Universal|
| Play-in-editor       | Preview mode: runtime simulation in viewport             |
| World settings       | Global scene settings: gravity, atmosphere, post-process |
| Blueprint-like nodes | Node editor for materials, procedural geometry           |
| Level streaming      | Scene partitioning for large worlds                      |

#### Borrowed from Rhino 3D

| Feature             | Adaptation in Forge                                       |
|---------------------|-----------------------------------------------------------|
| Command line        | Bottom bar: type commands like `box`, `extrude`, `trim`   |
| Layer panel         | Right panel tab: color-coded layers, visibility/lock toggles |
| Named views         | Front, Back, Left, Right, Top, Bottom, Perspective presets|
| Properties panel    | Tab in Details: NURBS/mesh info, surface degree, control points |
| Object snap (Osnap) | Snap toolbar: End, Mid, Center, Near, Perp, Tan, Quad, Int, Knot |
| Grasshopper nodes   | Node editor for materials, procedural content, state machines |
| Gumball manipulator | Combined translate/rotate/scale gizmo anchored to selection |
| Construction planes | User-defined work planes with grid                       |
| Make2D              | Generate 2D drawings from 3D geometry for export         |
| Analysis tools      | Curvature, draft angle, thickness, zebra analysis        |

#### Complete Interface Layout (ASCII)

```
+============================================================================+
|  [F] Forge    File  Edit  View  Insert  Modify  Analyze  Render  Help      |
+============================================================================+
|  [Select] [Move] [Rotate] [Scale] [Universal] || [Local/World] [Snap] [Grid]|
|  [Sketch] [Extrude] [Revolve] [Loft] [Sweep]  || [Undo] [Redo] | [Play]   |
+============================================================================+
|        |                                                    |              |
| BROWSER|              3D VIEWPORT                           | DETAILS      |
| -------+                                                    | ------------ |
| v Scene|  +---+                                             | Properties   |
|   v Grp |  | G |  (Gumball on selection)                    |  Transform   |
|     Cube|  +---+                                            |   Pos X [ 0 ]|
|     Cyl |                                                   |   Pos Y [ 0 ]|
|   Light1|         [Persp]  [Top]  [Front]  [Right]          |   Pos Z [ 0 ]|
|   Cam01 |                                                   |   Rot X [ 0 ]|
| --------+                                                   |   Rot Y [ 0 ]|
| LAYERS  |              Grid + scene objects                 |   Rot Z [ 0 ]|
| --------+                                                   |   Scl   [ 1 ]|
| [v][L] Layer 0  #e7e7ea                                     | ------------ |
| [v][L] Layer 1  #4eff93                                     | Material     |
| [v][ ] Layer 2  #3e55ff                                     |  [Crystalline]|
| [ ][L] Layer 3  #e94560                                     |  Albedo: ... |
|        |                                                    |  Rough:  0.5 |
| --------+                                                   | ------------ |
| COMPNTS |                                                   | OUTLINER     |
| --------+                                                   | ------------ |
| v Assembly1                                                 | > Scene Root |
|   v Body1                                                   |   > Group1   |
|     Sketch1                                                 |     Cube     |
|     Extrude1                                                |     Cylinder |
|   > Body2                                                   |   > Lights   |
|        |                                                    |     DirLight |
+--------+----------------------------------------------------+--------------+
|  CONTENT BROWSER                                                           |
|  [Models] [Materials] [Textures] [Audio] [Scripts] [Prefabs] [Scenes]      |
|  +------+ +------+ +------+ +------+ +------+ +------+ +------+           |
|  | Cube | | Wood | | Bark | | Step | | main | | Tree | | Lvl1 |           |
|  | .glb | | .mat | | .png | | .wav | | .lua | | .pfb | | .scn |           |
|  +------+ +------+ +------+ +------+ +------+ +------+ +------+           |
+============================================================================+
| TIMELINE   [|<] [<] [>] [>|]  ----[====o========]----  Frame: 24 / 120    |
|  [Sketch1]--[Extrude1]--[Fillet1]--[Mirror1]--[Joint1]        1.0s  30fps |
+============================================================================+
| > Command: box 2 3 4 _                    | Snap: End Mid Cen | Polys: 12k |
| Result: Box created (2x3x4) on Layer 0    | Grid: 1.0 | FPS: 144 | 48MB   |
+============================================================================+
```

**Panel Arrangement Details:**

- **Left Column** (240px default, resizable): Browser (scene tree), Layers, Components
- **Center**: 3D Viewport (fills remaining space), with named view tabs along top edge
- **Right Column** (280px default, resizable): Details/Properties, Outliner
- **Bottom Dock** (200px collapsed, expandable): Content Browser with thumbnail grid
- **Timeline Strip** (48px): Parametric history + animation timeline, docked below content browser
- **Command Bar** (dual-line, 40px): Command input + result, snap status, stats
- **Toolbar** (top, 36px x 2 rows): Frosted glass, mode selection and tool shortcuts
- **Menu Bar** (top, 28px): Standard application menu

All panels are dockable, floatable, tabbable, and support drag-to-rearrange. Double-click a panel header to maximize/restore.

---

### 1.3 Light Types, Render Styles, Asset Types Catalogs

#### Light Types

| #  | Light Type         | Description                                     | Stage |
|----|--------------------|-------------------------------------------------|-------|
| 1  | Directional        | Infinite parallel rays, sun simulation           | 1     |
| 2  | Point              | Omnidirectional from a single point              | 1     |
| 3  | Spot               | Cone-shaped with inner/outer angle               | 1     |
| 4  | Area (Rect)        | Rectangular emissive surface, soft shadows       | 2     |
| 5  | Area (Disc)        | Circular emissive surface                        | 2     |
| 6  | Hemisphere / Sky   | Ambient hemisphere with ground color              | 2     |
| 7  | IES Profile        | Photometric light from .ies files                | 4     |
| 8  | Emissive Mesh      | Any mesh surface as a light source               | 4     |
| 9  | HDRI Environment   | Image-based lighting from .hdr/.exr              | 2     |
| 10 | Volumetric (Fog)   | Light scattering through participating media     | 6     |
| 11 | Portal             | Guides importance sampling through windows       | 6     |
| 12 | Tube / Capsule     | Linear light source, fluorescent tube style      | 4     |

#### Render Styles

| #  | Render Style         | Description                                        | Stage |
|----|----------------------|----------------------------------------------------|-------|
| 1  | Solid (Lit)          | Standard PBR shading with all lights               | 1     |
| 2  | Wireframe            | Edge-only rendering                                | 1     |
| 3  | Solid + Wireframe    | Shaded with wireframe overlay                      | 1     |
| 4  | Unlit / Flat         | Albedo only, no lighting                           | 1     |
| 5  | Normals              | Visualize surface normals as RGB                   | 1     |
| 6  | UV Checker           | Checkered pattern to inspect UV mapping            | 2     |
| 7  | Depth                | Grayscale depth buffer visualization               | 2     |
| 8  | AO Only              | Ambient occlusion pass only                        | 2     |
| 9  | Matcap               | Material capture sphere mapping                    | 2     |
| 10 | Clay                 | Uniform gray diffuse, good for form evaluation     | 2     |
| 11 | X-Ray                | Semi-transparent with edge emphasis                | 3     |
| 12 | Curvature Analysis   | Color-coded curvature visualization                | 3     |
| 13 | Draft Angle          | Highlight undercuts for manufacturing              | 3     |
| 14 | Zebra Stripes        | Surface continuity analysis                        | 3     |
| 15 | Toon / Cel           | Non-photorealistic cel-shaded look                 | 4     |
| 16 | Thickness Analysis   | Color-mapped wall thickness                        | 5     |
| 17 | PBR Preview          | Final quality PBR with IBL and post-processing     | 6     |
| 18 | Path Traced          | Progressive path tracing for reference rendering   | 6     |

#### Asset Types

| #  | Asset Type     | Extensions            | Description                            | Stage |
|----|----------------|-----------------------|----------------------------------------|-------|
| 1  | Mesh           | `.glb`, `.gltf`       | 3D geometry (all meshes including primitives) | 1  |
| 2  | Texture        | `.png`, `.jpg`, `.exr`, `.hdr` | Image maps for materials        | 1     |
| 3  | Material       | `.mat.toml`           | Material definitions (serialized)      | 2     |
| 4  | Scene          | `.scene.toml`         | Scene graph serialization              | 2     |
| 5  | Prefab         | `.prefab.toml`        | Reusable entity templates              | 2     |
| 6  | Animation      | `.anim.toml`          | Keyframe animation data                | 4     |
| 7  | Skeleton       | `.skel.toml`          | Bone hierarchy                         | 4     |
| 8  | Audio          | `.wav`, `.ogg`        | Sound effects and music                | 5     |
| 9  | Script         | `.lua`                | Gameplay scripts                       | 5     |
| 10 | Font           | `.ttf`, `.otf`        | UI fonts                               | 1     |
| 11 | Shader         | `.wgsl`               | Custom shader source                   | 2     |
| 12 | Node Graph     | `.nodegraph.toml`     | Visual scripting / material graphs     | 3     |
| 13 | IES Profile    | `.ies`                | Photometric light data                 | 4     |
| 14 | Physics        | `.phys.toml`          | Collider and physics body definitions  | 4     |
| 15 | Terrain        | `.terrain.toml`       | Heightmap-based terrain data           | 3     |
| 16 | Particle       | `.particle.toml`      | Particle system definitions            | 5     |
| 17 | Project        | `.forgeproject.toml`  | Project manifest                       | 1     |
| 18 | Config         | `.config.toml`        | Editor and user preferences            | 1     |

---

### 1.4 Primitive Meshes (.glb)

All primitive meshes are stored as `.glb` files in `assets/primitives/`. They are loaded at startup and instanced in the scene.

| Primitive        | File                     | Description                         |
|------------------|--------------------------|-------------------------------------|
| Cube             | `cube.glb`               | Unit cube (1x1x1), origin at center|
| Sphere           | `sphere.glb`             | UV sphere, 32 segments, radius 0.5 |
| Ico Sphere       | `ico_sphere.glb`         | Icosphere, 3 subdivisions          |
| Cylinder         | `cylinder.glb`           | Radius 0.5, height 1, 32 segments  |
| Cone             | `cone.glb`               | Radius 0.5, height 1, 32 segments  |
| Torus            | `torus.glb`              | Major 0.5, minor 0.15, 32x16 segs  |
| Plane            | `plane.glb`              | 1x1, single quad, normal +Y        |
| Grid             | `grid.glb`               | 10x10 subdivided plane              |
| Capsule          | `capsule.glb`            | Radius 0.25, height 1, hemicaps    |
| Wedge            | `wedge.glb`              | Right-angle wedge (triangular prism)|
| Pyramid          | `pyramid.glb`            | Square base, 4 triangular faces     |
| Disc             | `disc.glb`               | Flat circle, radius 0.5, 32 segs   |
| Tube / Pipe      | `tube.glb`               | Hollow cylinder, wall thickness 0.05|
| Stairs           | `stairs.glb`             | 8-step staircase block              |
| Arch             | `arch.glb`               | Semicircular arch, 16 segments      |
| Spring / Helix   | `helix.glb`              | Helical coil, 4 turns              |
| Suzanne          | `suzanne.glb`            | Blender monkey for testing          |

---

### 1.5 Mouse Cursor Policy

The default cursor is **free** (standard OS pointer). Cursor lock is only engaged in FPS preview mode.

| Context                      | Cursor Style         | Lock   | Notes                              |
|------------------------------|----------------------|--------|------------------------------------|
| Default / Idle               | Arrow                | Free   | Standard OS pointer                |
| Hovering interactive UI      | Hand (pointer)       | Free   | Buttons, links, toggles            |
| Hovering viewport (idle)     | Crosshair            | Free   | Thin crosshair for precision       |
| Orbit (RMB held)             | Orbit icon           | Free   | Custom SVG orbit cursor            |
| Pan (MMB held)               | Grab (closed hand)   | Free   | Panning the view                   |
| Zoom (Scroll / RMB+LMB)     | Zoom icon            | Free   | Magnifying glass +/-               |
| FPS fly mode (Shift+RMB)     | Hidden               | Locked | Only locked mode; ESC to exit      |
| Translate gizmo hover        | Move arrows          | Free   | Axis-colored arrows                |
| Rotate gizmo hover           | Rotate icon          | Free   | Circular arrows                    |
| Scale gizmo hover            | Scale icon           | Free   | Bidirectional arrows               |
| Box select (drag)            | Crosshair            | Free   | Selection rectangle                |
| Lasso select                 | Lasso icon           | Free   | Freeform selection                 |
| Sketch mode drawing          | Pencil               | Free   | 2D sketch context                  |
| Text input / command line    | I-beam               | Free   | Text editing contexts              |
| Resize panel border          | Col-resize / Row-resize | Free | Dragging panel edges              |
| Drag-and-drop asset          | Grab + thumbnail     | Free   | Dragging from content browser      |
| Node editor wiring           | Crosshair + wire     | Free   | Dragging a connection wire         |
| Color picker eyedropper      | Eyedropper           | Free   | Sampling color from viewport       |
| Measurement tool             | Crosshair + ruler    | Free   | Point-to-point measurement         |
| Disabled / loading           | Wait (spinner)       | Free   | During heavy computation           |

---

## Part 2: Architecture

### 2.1 MVVM Pattern

```
┌───────────────────────────────────────────────────────────────────────┐
│                           MVVM in Forge                              │
├───────────────────────────────────────────────────────────────────────┤
│                                                                       │
│   ┌─────────┐       ┌──────────────┐        ┌─────────┐             │
│   │  VIEW   │◄─────►│  VIEW-MODEL  │◄──────►│  MODEL  │             │
│   │ (egui)  │ binds │ (State +     │ reads/ │ (Core   │             │
│   │         │       │  Commands)   │ writes │  Data)  │             │
│   └─────────┘       └──────────────┘        └─────────┘             │
│       │                    │                      │                   │
│       │ renders            │ transforms           │ persists          │
│       │ UI panels          │ data for display     │ entities          │
│       │ handles input      │ validates input      │ components        │
│       │ emits actions      │ executes commands    │ resources         │
│       │                    │ manages undo/redo    │ assets            │
│       ▼                    ▼                      ▼                   │
│  ┌──────────┐     ┌──────────────┐        ┌───────────┐             │
│  │  egui    │     │  Command Bus │        │   ECS     │             │
│  │  Panels  │     │  + Event     │        │  World    │             │
│  │  Widgets │     │  Dispatcher  │        │  Storage  │             │
│  └──────────┘     └──────────────┘        └───────────┘             │
│                                                                       │
│   Data Flow:                                                          │
│   User Input → View → Action → ViewModel → Command → Model → Notify  │
│   Model Change → ViewModel (observe) → View (rebind) → Re-render     │
│                                                                       │
│   Key Principles:                                                     │
│   - Views never access Model directly                                 │
│   - ViewModels are testable without UI                                │
│   - Commands are undoable, serializable                               │
│   - All state mutations go through the Command Bus                    │
│   - Reactive bindings via change notification channels                │
└───────────────────────────────────────────────────────────────────────┘
```

**Layer Responsibilities:**

| Layer      | Crate(s)                         | Responsibility                                |
|------------|----------------------------------|-----------------------------------------------|
| Model      | `core`, `scene`, `assets`, `physics`, `animation` | Domain data, ECS world, business rules |
| ViewModel  | `viewport`, `inspector`, `modeling`, `materials`, `scripting` | State projection, commands, validation |
| View       | `ui`, `app`                      | egui panels, input routing, rendering dispatch |
| Services   | `render`, `plugins`, `project`   | Cross-cutting: GPU, plugin host, persistence  |

### 2.2 Workspace Layout

```
forge-editor/
├── Cargo.toml                  # Workspace manifest
├── Cargo.lock
├── .cargo/
│   └── config.toml             # Build settings, linker flags
├── assets/
│   ├── primitives/             # .glb primitive meshes
│   ├── fonts/                  # Inter, JetBrains Mono
│   ├── icons/                  # SVG icon set
│   ├── themes/                 # .theme.toml files
│   └── shaders/                # .wgsl shader sources
├── crates/
│   ├── core/                   # [Model] ECS, math, events, undo, commands
│   ├── render/                 # [Service] wgpu renderer, passes, pipeline
│   ├── viewport/               # [ViewModel] Camera, gizmos, selection, snaps
│   ├── ui/                     # [View] egui panels, widgets, theme engine
│   ├── scene/                  # [Model] Scene graph, serialization, prefabs
│   ├── assets/                 # [Model] Asset pipeline, import, thumbnails
│   ├── inspector/              # [ViewModel] Property editor, detail views
│   ├── modeling/               # [ViewModel] Mesh ops, sketch, constraints
│   ├── materials/              # [ViewModel] Material graph, shader compiler
│   ├── animation/              # [Model] Keyframes, curves, skeletal, timeline
│   ├── physics/                # [Model] Rigid body, colliders, simulation
│   ├── scripting/              # [ViewModel] Lua VM, script editor, hot-reload
│   ├── plugins/                # [Service] Plugin host, sandboxed API, registry
│   └── project/                # [Service] Project manifest, VCS, export
├── app/                        # [View] Binary: window, main loop, integration
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── tools/                      # Build scripts, asset processors
├── docs/                       # Internal documentation
└── tests/                      # Integration tests
```

**Workspace Cargo.toml:**

```toml
[workspace]
resolver = "2"
members = [
    "crates/core",
    "crates/render",
    "crates/viewport",
    "crates/ui",
    "crates/scene",
    "crates/assets",
    "crates/inspector",
    "crates/modeling",
    "crates/materials",
    "crates/animation",
    "crates/physics",
    "crates/scripting",
    "crates/plugins",
    "crates/project",
    "app",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
rust-version = "1.85"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# GPU / Rendering
wgpu = "24.0"
naga = "24.0"

# UI
egui = "0.31"
eframe = "0.31"
egui-wgpu = "0.31"
egui-winit = "0.31"

# Windowing
winit = "0.30"

# Math
glam = { version = "0.29", features = ["mint", "serde"] }
ultraviolet = "0.9"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
ron = "0.8"

# Asset Loading
gltf = "1.4"
image = "0.25"

# ECS
hecs = "0.10"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Error Handling
anyhow = "1.0"
thiserror = "2.0"

# File System
notify = "7.0"
walkdir = "2.0"

# Scripting
mlua = { version = "0.10", features = ["luau", "serialize"] }

# Physics
rapier3d = "0.22"

# Async
tokio = { version = "1.44", features = ["rt-multi-thread", "fs", "sync"] }
pollster = "0.4"

# UUID
uuid = { version = "1.16", features = ["v4", "serde"] }

# Time
instant = "0.1"

# Misc
parking_lot = "0.12"
crossbeam-channel = "0.5"
indexmap = { version = "2.7", features = ["serde"] }
bitflags = "2.9"
bytemuck = { version = "1.21", features = ["derive"] }
smallvec = "1.14"
rfd = "0.15"

[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3

[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"
```

### 2.3 Core Dependencies

| Crate                | Version  | Purpose                                  |
|----------------------|----------|------------------------------------------|
| `wgpu`               | 24.0     | WebGPU-based rendering backend           |
| `naga`               | 24.0     | Shader translation and validation        |
| `egui`               | 0.31     | Immediate mode GUI framework             |
| `eframe`             | 0.31     | egui framework integration               |
| `egui-wgpu`          | 0.31     | egui wgpu rendering backend              |
| `egui-winit`         | 0.31     | egui winit input integration             |
| `winit`              | 0.30     | Cross-platform windowing                 |
| `glam`               | 0.29     | Linear algebra (Vec3, Mat4, Quat)        |
| `serde`              | 1.0      | Serialization framework                  |
| `toml`               | 0.8      | TOML config parsing                      |
| `gltf`               | 1.4      | glTF/GLB mesh loading                    |
| `image`              | 0.25     | Image loading and processing             |
| `hecs`               | 0.10     | Lightweight ECS                          |
| `tracing`            | 0.1      | Structured logging and diagnostics       |
| `tracing-subscriber` | 0.3      | Log output subscriber                    |
| `anyhow`             | 1.0      | Flexible error handling                  |
| `thiserror`          | 2.0      | Typed error derive macro                 |
| `notify`             | 7.0      | File system watcher (hot-reload)         |
| `walkdir`            | 2.0      | Recursive directory traversal            |
| `mlua`               | 0.10     | Lua/Luau scripting engine                |
| `rapier3d`           | 0.22     | 3D physics engine                        |
| `tokio`              | 1.44     | Async runtime                            |
| `pollster`           | 0.4      | Blocking on futures                      |
| `uuid`               | 1.16     | Unique identifiers                       |
| `parking_lot`        | 0.12     | Fast synchronization primitives          |
| `crossbeam-channel`  | 0.5      | Multi-producer multi-consumer channels   |
| `indexmap`           | 2.7      | Insertion-ordered hash map               |
| `bitflags`           | 2.9      | Bitflag type macro                       |
| `bytemuck`           | 1.21     | Safe transmutes for GPU data             |
| `smallvec`           | 1.14     | Stack-allocated small vectors            |
| `rfd`                | 0.15     | Native file dialogs                      |
| `ron`                | 0.8      | Rusty Object Notation                    |

### 2.4 Crate Dependency Graph

```
                            ┌─────────┐
                            │   app   │
                            └────┬────┘
                  ┌──────────────┼──────────────┐
                  │              │              │
             ┌────▼───┐    ┌────▼────┐   ┌─────▼─────┐
             │   ui   │    │ project │   │  plugins  │
             └────┬───┘    └────┬────┘   └─────┬─────┘
                  │             │               │
        ┌─────┬──┴──┬──────────┤               │
        │     │     │          │               │
   ┌────▼──┐ │  ┌──▼──────┐   │          ┌────▼─────┐
   │viewprt│ │  │inspector│   │          │scripting │
   └───┬───┘ │  └────┬────┘   │          └────┬─────┘
       │     │       │        │               │
       │  ┌──▼──────┐│   ┌────▼────┐          │
       │  │materials││   │animation│          │
       │  └────┬────┘│   └────┬────┘          │
       │       │     │        │               │
       │  ┌────▼───┐ │   ┌────▼───┐           │
       │  │modeling│ │   │physics │           │
       │  └────┬───┘ │   └────┬───┘           │
       │       │     │        │               │
       ├───────┴─────┼────────┘               │
       │             │                        │
  ┌────▼───┐    ┌────▼───┐                    │
  │ render │    │ assets │                    │
  └────┬───┘    └────┬───┘                    │
       │             │                        │
  ┌────▼───┐    ┌────▼───┐                    │
  │ scene  │    │        │                    │
  └────┬───┘    │        │                    │
       │        │        │                    │
       └────┬───┘        │                    │
            │            │                    │
       ┌────▼───┐        │                    │
       │  core  │◄───────┴────────────────────┘
       └────────┘
```

**Dependency Rules:**
- `core` depends on nothing internal (only external crates)
- `scene` depends on `core`
- `render` depends on `core`, `scene`
- `assets` depends on `core`, `scene`
- `viewport` depends on `core`, `scene`, `render`
- `modeling` depends on `core`, `scene`
- `materials` depends on `core`, `scene`, `render`, `assets`
- `inspector` depends on `core`, `scene`, `assets`
- `animation` depends on `core`, `scene`
- `physics` depends on `core`, `scene`
- `scripting` depends on `core`, `scene`
- `plugins` depends on `core`, `scripting`
- `ui` depends on `core`, `scene`, `render`, `viewport`, `inspector`, `modeling`, `materials`
- `project` depends on `core`, `scene`, `assets`
- `app` depends on all crates (integration point)

---

## Part 3: Staged Roadmap

### Stage 1: Foundation MVP (Weeks 1-6)

**Goal:** Open a window with a 3D viewport, render a grid and a primitive cube, orbit camera, basic egui chrome. Establish the MVVM command infrastructure.

#### Crate: `core` [Model Layer]

**Cargo.toml:**
```toml
[package]
name = "core"
version.workspace = true
edition.workspace = true

[dependencies]
glam = { workspace = true }
serde = { workspace = true }
uuid = { workspace = true }
hecs = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
parking_lot = { workspace = true }
crossbeam-channel = { workspace = true }
indexmap = { workspace = true }
bitflags = { workspace = true }
bytemuck = { workspace = true }
smallvec = { workspace = true }
toml = { workspace = true }
```

**TODO Checklist:**
- [ ] Define `EntityId` as newtype over `uuid::Uuid`
- [ ] Define `Transform` struct (position: Vec3, rotation: Quat, scale: Vec3) with `local_matrix()` and `world_matrix()` methods
- [ ] Implement `TransformHierarchy` for parent-child world matrix computation
- [ ] Define ECS `World` wrapper over `hecs::World` with typed accessors
- [ ] Define `Component` trait for serializable components
- [ ] Implement `Name`, `Visibility`, `Tag`, `Layer` components
- [ ] Define `Command` trait: `execute(&mut World)`, `undo(&mut World)`, `description() -> &str`
- [ ] Implement `CommandBus` with undo/redo stacks (max 256 entries)
- [ ] Implement `EventBus` with typed channels via `crossbeam-channel`
- [ ] Define core events: `EntityCreated`, `EntityDeleted`, `ComponentChanged`, `SelectionChanged`
- [ ] Implement `Selection` struct: ordered set of `EntityId`, primary selection, multi-select
- [ ] Implement `LayerSystem`: layer id, name, color, visible, locked, layer membership per entity
- [ ] Define `BoundingBox` (AABB) with union, intersection, contains, ray-test
- [ ] Define `Ray` struct with `origin`, `direction`, `intersect_aabb`, `intersect_plane`, `intersect_triangle`
- [ ] Implement `IdGenerator` for unique sequential naming ("Cube", "Cube.001", "Cube.002")
- [ ] Define `Units` enum (Millimeters, Centimeters, Meters, Inches, Feet) with conversion table
- [ ] Define `ColorRGBA` struct (f32 x 4) with hex parsing, sRGB/linear conversion
- [ ] Define `ThemeConfig` struct deserializable from `.theme.toml`
- [ ] Implement `load_theme(path) -> ThemeConfig` with default Crystalline fallback
- [ ] Implement `ChangeNotifier` for MVVM reactive binding (notify ViewModel of Model changes)
- [ ] Write unit tests for Transform, BoundingBox, Ray, CommandBus undo/redo, Selection
- [ ] Define `AppConfig` struct for editor preferences (recent files, last layout, etc.)
- [ ] Implement `Clipboard` struct for cut/copy/paste of entities
- [ ] Add `serde` Serialize/Deserialize for all core types

**Acceptance Criteria:**
- All core types compile and pass unit tests
- CommandBus supports execute, undo, redo with proper stack management
- EventBus delivers typed messages to all subscribers
- Transform hierarchy computes correct world matrices for 3-level nesting
- Theme loads from TOML and provides all color values

---

#### Crate: `render` [Service Layer]

**Cargo.toml:**
```toml
[package]
name = "render"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
wgpu = { workspace = true }
naga = { workspace = true }
glam = { workspace = true }
bytemuck = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
image = { workspace = true }
pollster = { workspace = true }
parking_lot = { workspace = true }
```

**TODO Checklist:**
- [ ] Initialize `wgpu::Instance`, `Adapter`, `Device`, `Queue` with error handling and fallback to software rasterizer
- [ ] Create `RenderContext` struct holding device, queue, surface configuration
- [ ] Implement `SurfaceManager` for window surface creation and resize handling
- [ ] Define `Vertex` struct: position (Vec3), normal (Vec3), uv (Vec2), color (Vec4) with `bytemuck` derive
- [ ] Implement `MeshBuffer` struct: vertex buffer, index buffer, vertex count, index count
- [ ] Write `upload_mesh(device, vertices, indices) -> MeshBuffer`
- [ ] Implement `CameraUniform` struct with view, projection, view_projection matrices
- [ ] Create `UniformBuffer<T>` generic wrapper for GPU uniform data
- [ ] Write WGSL shader: `solid.wgsl` with PBR-lite (Blinn-Phong) for Stage 1
- [ ] Write WGSL shader: `wireframe.wgsl` for edge rendering using line primitives
- [ ] Write WGSL shader: `grid.wgsl` for infinite ground grid with fade at horizon
- [ ] Write WGSL shader: `gizmo.wgsl` for unlit, always-on-top gizmo rendering
- [ ] Implement `RenderPipeline` builder wrapping `wgpu::RenderPipeline` with sensible defaults
- [ ] Implement `RenderPass` trait with `prepare()` and `execute()` methods
- [ ] Implement `GridPass`: renders XZ ground grid with major/minor lines in theme colors
- [ ] Implement `SolidPass`: renders all opaque meshes with basic lighting
- [ ] Implement `WireframePass`: renders wireframe overlay
- [ ] Implement `GizmoPass`: renders translate/rotate/scale gizmos on top
- [ ] Implement `SelectionPass`: renders selection outline and glow as post-process
- [ ] Implement `FrameGraph` to orchestrate pass execution order
- [ ] Implement `DepthBuffer` creation and management
- [ ] Create `ViewportRenderTarget` for rendering to texture (for egui embedding)
- [ ] Implement viewport background gradient shader (top `#0a0a14` to bottom `#1a1a2e`)
- [ ] Add directional light struct and pass as uniform to solid shader
- [ ] Write unit tests for uniform buffer upload, vertex layout

**Acceptance Criteria:**
- A textured or lit cube renders correctly in a wgpu surface
- Grid renders with correct theme colors and fades at distance
- Wireframe mode toggles on/off
- Gizmos render on top of scene geometry
- Viewport resizes without crashing

---

#### Crate: `viewport` [ViewModel Layer]

**Cargo.toml:**
```toml
[package]
name = "viewport"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
render = { path = "../render" }
glam = { workspace = true }
winit = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
serde = { workspace = true }
```

**TODO Checklist:**
- [ ] Implement `Camera` struct: position, target, up, fov, near, far, aspect
- [ ] Implement `OrbitController`: right-click drag to orbit around target, scroll to zoom, MMB to pan
- [ ] Implement `FlyController`: WASD + mouse look (Shift+RMB to engage), speed control with scroll
- [ ] Implement `CameraController` enum dispatching to Orbit or Fly
- [ ] Implement smooth camera transitions (lerp position/target over N frames)
- [ ] Define `NamedView` enum: Front, Back, Left, Right, Top, Bottom, Perspective, Custom(String)
- [ ] Implement `snap_to_view(NamedView)` with smooth animation
- [ ] Implement `TranslateGizmo`: 3 axis arrows + 3 plane squares + center cube
- [ ] Implement `RotateGizmo`: 3 axis rings + screen-space ring + trackball
- [ ] Implement `ScaleGizmo`: 3 axis cubes + uniform scale center
- [ ] Implement `UniversalGizmo`: combined translate + rotate + scale (Gumball style)
- [ ] Implement gizmo hit-testing via ray intersection with gizmo geometry
- [ ] Implement `GizmoInteraction`: drag logic projecting mouse delta onto constrained axis/plane
- [ ] Implement coordinate space toggle: Local / World / Screen
- [ ] Implement `SelectionRaycast`: pick entities by casting ray through mouse position
- [ ] Implement `BoxSelect`: drag rectangle to select all entities within frustum
- [ ] Implement `ObjectSnapSystem`: End, Mid, Center, Near, Perp, Tan, Quad, Int, Knot, Grid
- [ ] Implement snap point detection and visual indicator rendering
- [ ] Implement `GridSnap`: configurable grid size, snap cursor to nearest grid point
- [ ] Emit `SelectionChanged`, `TransformChanged` events through `CommandBus`
- [ ] Implement `ViewportState` ViewModel: current tool, gizmo mode, snap settings, camera state
- [ ] Implement `FocusOnSelection`: frame selected objects in viewport (F key)
- [ ] Implement cursor management: set cursor icon based on context (see 1.5 table)
- [ ] Handle DPI scaling for gizmo sizes and pick thresholds

**Acceptance Criteria:**
- Orbit, pan, zoom work smoothly
- Named views snap to correct orientations
- Gizmos display on selection and respond to drag
- Ray-pick selects the nearest entity under cursor
- Object snaps display indicator dots at snap points

---

#### Crate: `ui` [View Layer]

**Cargo.toml:**
```toml
[package]
name = "ui"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
render = { path = "../render" }
viewport = { path = "../viewport" }
egui = { workspace = true }
eframe = { workspace = true }
egui-wgpu = { workspace = true }
glam = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
rfd = { workspace = true }
```

**TODO Checklist:**
- [ ] Implement `ThemeEngine`: load `ThemeConfig`, convert to egui `Visuals` and `Style`
- [ ] Implement frosted glass panel rendering via custom `egui::Frame` with blur approximation
- [ ] Implement `MenuBar` panel: File, Edit, View, Insert, Modify, Analyze, Render, Help menus
- [ ] Implement `Toolbar` panel: tool selection buttons with morphing hover effect
- [ ] Implement `StatusBar` panel: coordinate readout, snap toggles, polygon count, FPS, memory
- [ ] Implement `OutlinerPanel`: scene hierarchy tree with expand/collapse, drag reorder, type icons
- [ ] Implement `LayerPanel`: layer list with color swatch, visibility eye icon, lock icon
- [ ] Implement `ViewportPanel`: embeds the wgpu render target as an egui `Image`/`TextureId`
- [ ] Implement viewport overlay HUD: current named view label, axis indicator gizmo in corner
- [ ] Implement `CommandLinePanel`: text input at bottom, auto-complete dropdown, result display
- [ ] Implement `DockingLayout`: save/restore panel arrangement to JSON/TOML
- [ ] Implement panel resizing by dragging borders
- [ ] Implement panel tab groups (drag panel header onto another to create tabs)
- [ ] Implement right-click context menus for scene entities
- [ ] Implement keyboard shortcut system (configurable, saved in config)
- [ ] Implement `MorphingButton` widget: gradient shifts based on hover/press state
- [ ] Implement `ColorSwatch` widget: small colored square with tooltip
- [ ] Implement `CollapsibleSection` widget: header with expand arrow, smooth animation
- [ ] Implement `SearchBar` widget with filter-as-you-type for outliner and content browser
- [ ] Implement toast notification system (success/warning/error) in bottom-right corner
- [ ] Implement modal dialog system for confirmations and file operations
- [ ] Implement initial file dialog integration (rfd) for Open/Save
- [ ] Write shortcut defaults: Ctrl+Z undo, Ctrl+Y redo, Ctrl+S save, Delete remove, F focus, G grab, R rotate, S scale
- [ ] Implement dark/light mode toggle in View menu

**Acceptance Criteria:**
- Full chrome renders with all panels visible
- Panels resize and can be rearranged
- Theme applies consistently across all widgets
- Command line accepts text input and displays results
- Keyboard shortcuts trigger correct actions

---

#### Crate: `app` [View Layer - Binary]

**Cargo.toml:**
```toml
[package]
name = "app"
version.workspace = true
edition.workspace = true

[[bin]]
name = "forge"
path = "src/main.rs"

[dependencies]
core = { path = "../crates/core" }
render = { path = "../crates/render" }
viewport = { path = "../crates/viewport" }
ui = { path = "../crates/ui" }
scene = { path = "../crates/scene" }
assets = { path = "../crates/assets" }
inspector = { path = "../crates/inspector" }
modeling = { path = "../crates/modeling" }
materials = { path = "../crates/materials" }
animation = { path = "../crates/animation" }
physics = { path = "../crates/physics" }
scripting = { path = "../crates/scripting" }
plugins = { path = "../crates/plugins" }
project = { path = "../crates/project" }
winit = { workspace = true }
wgpu = { workspace = true }
egui = { workspace = true }
eframe = { workspace = true }
egui-wgpu = { workspace = true }
egui-winit = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
pollster = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
```

**TODO Checklist:**
- [ ] Initialize `tracing-subscriber` with env filter and structured output
- [ ] Create `winit` event loop and window with title "Forge Editor"
- [ ] Set window icon and minimum size (1024x768)
- [ ] Initialize `RenderContext` from `render` crate
- [ ] Initialize `ECS World` from `core`
- [ ] Initialize `ThemeEngine` loading `crystalline.theme.toml`
- [ ] Initialize `CommandBus` and `EventBus`
- [ ] Initialize `Camera` with default orbit at origin
- [ ] Spawn default scene: directional light + ground grid
- [ ] Set up `eframe` integration with custom `wgpu` backend
- [ ] Wire input events from `winit` to `viewport` and `ui`
- [ ] Implement main loop: process input -> update ViewModels -> execute commands -> render -> present
- [ ] Handle window resize: update surface, camera aspect, egui viewport
- [ ] Implement graceful shutdown: save layout, flush logs
- [ ] Handle DPI change events
- [ ] Implement command-line argument parsing: `--project <path>`, `--theme <name>`, `--verbose`
- [ ] Implement startup splash screen with Forge logo (optional, toggle in config)
- [ ] Add panic hook with crash report dialog
- [ ] Profile frame time and display in status bar

**Acceptance Criteria:**
- Application launches, shows a window with 3D viewport and egui chrome
- Default cube or empty scene with grid is visible
- Camera orbit/pan/zoom works
- egui panels are interactive
- Clean shutdown without errors

---

### Stage 2: Scene & Assets (Weeks 7-12)

**Goal:** Full scene graph with save/load, asset import pipeline with .glb loading, content browser with thumbnails, and property inspector.

#### Crate: `scene` [Model Layer]

**Cargo.toml:**
```toml
[package]
name = "scene"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
glam = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
uuid = { workspace = true }
hecs = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
indexmap = { workspace = true }
```

**TODO Checklist:**
- [ ] Define `SceneGraph` struct: tree of `SceneNode` with parent/children references via `EntityId`
- [ ] Implement `SceneNode`: entity_id, name, transform, children vec, parent option, visible, locked
- [ ] Implement `SceneGraph::add_child`, `remove_child`, `reparent`, `traverse_depth_first`
- [ ] Implement world transform computation: walk up parent chain multiplying local transforms
- [ ] Define `Prefab` struct: a serializable sub-tree of the scene graph
- [ ] Implement `instantiate_prefab(prefab, parent, position) -> Vec<EntityId>`
- [ ] Implement `create_prefab_from_selection(selection) -> Prefab`
- [ ] Define `SceneFile` struct: metadata + serialized scene graph + component data
- [ ] Implement `save_scene(world, path)` serializing to `.scene.toml`
- [ ] Implement `load_scene(path) -> SceneFile` deserializing and spawning entities
- [ ] Implement `Group` component: logical grouping without transform hierarchy
- [ ] Implement `Duplicate` command: deep clone selected entities with new IDs and offset names
- [ ] Implement `DeleteEntities` command: remove selected entities and their children
- [ ] Implement `ReparentEntities` command: move entities to new parent
- [ ] Implement scene-level metadata: name, author, units, up-axis, creation date
- [ ] Implement `SceneDirtyFlag`: track unsaved changes, prompt on close
- [ ] Define `MeshComponent`: reference to mesh asset, material slots, render flags
- [ ] Define `LightComponent`: light type enum, color, intensity, range, inner/outer angles
- [ ] Define `CameraComponent`: fov, near, far, target, is_active
- [ ] Implement `SceneQuery`: find entities by name, component type, layer, tag
- [ ] Write unit tests for scene graph operations, serialization round-trip

**Acceptance Criteria:**
- Scene saves to TOML and loads back identically
- Parent-child hierarchy works correctly through reparenting
- Prefabs instantiate with correct transforms
- Dirty flag tracks modifications

---

#### Crate: `assets` [Model Layer]

**Cargo.toml:**
```toml
[package]
name = "assets"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
gltf = { workspace = true }
image = { workspace = true }
glam = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
notify = { workspace = true }
walkdir = { workspace = true }
parking_lot = { workspace = true }
tokio = { workspace = true }
```

**TODO Checklist:**
- [ ] Define `AssetId` as newtype over `Uuid`
- [ ] Define `AssetHandle<T>` as a ref-counted smart pointer to loaded assets
- [ ] Implement `AssetRegistry`: HashMap<AssetId, AssetMeta> for all known assets
- [ ] Implement `AssetMeta`: id, path, type, size, last_modified, thumbnail_path
- [ ] Implement `AssetLoader` trait with `load(path) -> Result<Asset>` and `extensions() -> &[&str]`
- [ ] Implement `GlbLoader`: parse `.glb`/`.gltf` files into mesh vertices/indices + materials
- [ ] Implement `ImageLoader`: parse `.png`/`.jpg`/`.exr`/`.hdr` into texture data
- [ ] Implement `AssetManager`: lazy loading, caching, reference counting, unload unused
- [ ] Implement `AssetImporter`: copy external files into project `assets/` directory, assign AssetId
- [ ] Implement `ThumbnailGenerator`: render 128x128 previews of meshes and materials
- [ ] Implement `AssetWatcher` via `notify`: watch project asset directory for changes
- [ ] Handle asset hot-reload: detect file change, reload asset, update all references
- [ ] Implement `PrimitiveMeshLoader`: load all `assets/primitives/*.glb` at startup
- [ ] Define `MeshAsset`: vertices, indices, sub-meshes, bounding box, vertex count, triangle count
- [ ] Define `TextureAsset`: pixel data, dimensions, format, mip levels
- [ ] Define `MaterialAsset`: albedo, metallic, roughness, normal, emissive textures + scalar values
- [ ] Implement `ContentBrowserModel`: folder tree, file list, search, filter by type
- [ ] Implement drag-and-drop from content browser to viewport (spawn entity at drop point)
- [ ] Implement asset deletion with reference checking (warn if asset is in use)
- [ ] Write integration tests for GLB loading, image loading, hot-reload

**Acceptance Criteria:**
- GLB files load and render correctly in the viewport
- Content browser shows asset thumbnails
- Hot-reload updates meshes/textures without restart
- Primitives are available from the content browser
- Drag-and-drop spawns entities at correct positions

---

#### Crate: `inspector` [ViewModel Layer]

**Cargo.toml:**
```toml
[package]
name = "inspector"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
assets = { path = "../assets" }
glam = { workspace = true }
egui = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
```

**TODO Checklist:**
- [ ] Implement `InspectorViewModel`: reads selected entity, exposes component data for editing
- [ ] Implement `TransformEditor`: position, rotation (euler), scale fields with drag-edit and text input
- [ ] Implement `NameEditor`: editable text field for entity name
- [ ] Implement `VisibilityToggle`: eye icon toggle
- [ ] Implement `LayerSelector`: dropdown to assign entity layer
- [ ] Implement `MeshComponentEditor`: display vertex/tri count, bounding box, material slot list
- [ ] Implement `LightComponentEditor`: type selector, color picker, intensity slider, range, angle
- [ ] Implement `CameraComponentEditor`: fov slider, near/far, active toggle, preview window
- [ ] Implement `MaterialEditor`: albedo color picker, texture slots, PBR sliders (metallic, roughness)
- [ ] Implement generic property grid: auto-generate editors from serde-reflected types
- [ ] Implement multi-selection editing: show common properties, blank differing values
- [ ] Implement `AddComponentButton`: dropdown to attach new components to selected entity
- [ ] Implement `RemoveComponentButton`: right-click context menu option
- [ ] All edits emit commands through `CommandBus` for undo/redo support
- [ ] Implement copy/paste component values between entities
- [ ] Implement reset-to-default button per property (right-click -> Reset)
- [ ] Display component in collapsible sections with icons
- [ ] Implement search/filter within inspector properties
- [ ] Handle inspector update when selection changes or component data changes externally

**Acceptance Criteria:**
- Selecting an entity shows its properties in the inspector
- Editing a property emits a command and updates the viewport
- Undo/redo works for all inspector edits
- Multi-select shows shared properties
- Adding/removing components works

---

### Stage 3: Editing Tools (Weeks 13-20)

**Goal:** Mesh modeling operations, sketch mode, material graph editor, terrain system, and advanced viewport features.

#### Crate: `modeling` [ViewModel Layer]

**Cargo.toml:**
```toml
[package]
name = "modeling"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
glam = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
smallvec = { workspace = true }
```

**TODO Checklist:**
- [ ] Implement `HalfEdgeMesh` data structure for topological mesh editing
- [ ] Implement mesh conversion: glTF triangle list <-> HalfEdgeMesh
- [ ] Implement vertex/edge/face selection modes
- [ ] Implement `ExtrudeCommand`: extrude selected faces along normal or direction
- [ ] Implement `InsetCommand`: inset selected faces
- [ ] Implement `BevelCommand`: bevel selected edges with segments parameter
- [ ] Implement `LoopCutCommand`: insert edge loop at specified position
- [ ] Implement `SubdivideCommand`: Catmull-Clark subdivision
- [ ] Implement `BooleanCommand`: union, subtract, intersect two meshes
- [ ] Implement `MirrorCommand`: mirror mesh across specified plane
- [ ] Implement `ArrayCommand`: linear/radial array duplication
- [ ] Implement `BridgeEdgesCommand`: connect two edge loops
- [ ] Implement `MergeVerticesCommand`: weld vertices within threshold
- [ ] Implement `SmoothCommand`: Laplacian smoothing on selected vertices
- [ ] Implement `SketchMode`: enter/exit 2D drawing on construction plane
- [ ] Implement sketch primitives: Line, Arc, Circle, Rectangle, Polygon, Spline
- [ ] Implement sketch constraints: Coincident, Parallel, Perpendicular, Tangent, Equal, Horizontal, Vertical, Fixed, Dimension
- [ ] Implement sketch solver: resolve constraints to find geometry positions
- [ ] Implement `ExtrudeSketchCommand`: extrude closed sketch profile into 3D solid
- [ ] Implement `RevolveSketchCommand`: revolve sketch profile around axis
- [ ] Implement `LoftCommand`: loft between multiple sketch profiles
- [ ] Implement `SweepCommand`: sweep profile along path curve
- [ ] Implement `FilletCommand`: fillet selected edges with radius
- [ ] Implement `ChamferCommand`: chamfer selected edges with distance
- [ ] Implement `TerrainSystem`: heightmap-based terrain with brush sculpt/paint tools
- [ ] Define `TerrainComponent`: resolution, size, height data, splat map, texture layers

**Acceptance Criteria:**
- Extrude, bevel, loop cut work on mesh faces/edges
- Sketch mode allows drawing constrained 2D profiles
- Sketches extrude into 3D geometry with parametric history
- Boolean operations produce correct results
- Terrain creates and sculpts via brushes

---

#### Crate: `materials` [ViewModel Layer]

**Cargo.toml:**
```toml
[package]
name = "materials"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
render = { path = "../render" }
assets = { path = "../assets" }
glam = { workspace = true }
serde = { workspace = true }
toml = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
uuid = { workspace = true }
indexmap = { workspace = true }
```

**TODO Checklist:**
- [ ] Define `MaterialGraph` struct: DAG of `MaterialNode` connected by `Wire`
- [ ] Define `MaterialNode` trait: inputs, outputs, evaluate, generate WGSL snippet
- [ ] Implement PBR output node: Albedo, Metallic, Roughness, Normal, Emissive, AO, Opacity
- [ ] Implement texture sample node: UV input, texture asset reference, output RGBA
- [ ] Implement math nodes: Add, Subtract, Multiply, Divide, Power, Lerp, Clamp, Smoothstep
- [ ] Implement vector nodes: Split, Combine, Normalize, Dot, Cross, Transform
- [ ] Implement color nodes: HSV<->RGB, Brightness/Contrast, Invert, Overlay, Gamma
- [ ] Implement procedural nodes: Noise (Perlin, Simplex, Worley), Gradient, Checker, Brick
- [ ] Implement UV nodes: Tiling, Offset, Rotate, Projection (Planar, Box, Cylindrical, Spherical)
- [ ] Implement constant nodes: Float, Vec2, Vec3, Vec4, Color
- [ ] Implement time node for animated materials
- [ ] Implement `MaterialCompiler`: traverse graph, generate WGSL fragment shader
- [ ] Implement `NodeEditorViewModel`: node positions, wire routing, selection, pan/zoom
- [ ] Implement node creation: right-click context menu with categorized node list
- [ ] Implement wire drawing: click output pin, drag to input pin, type-checking
- [ ] Implement node preview: small render preview on each node showing output
- [ ] Implement material preview sphere in inspector when material is selected
- [ ] Implement material hot-swap: recompile shader on graph change, update in viewport
- [ ] Implement material library: save/load material presets
- [ ] Implement material assignment: drag material to mesh or assign via inspector
- [ ] Write tests for graph evaluation and WGSL generation

**Acceptance Criteria:**
- Node editor renders with Grasshopper-style nodes and wires
- Connecting nodes generates valid WGSL shaders
- Materials preview correctly on meshes in the viewport
- Material presets save and load
- Procedural textures animate when time node is connected

---

**Additional Stage 3 Features (implemented across existing crates):**

- [ ] `render`: Add UV checker, depth, AO, matcap, clay render styles
- [ ] `render`: Implement X-ray, curvature analysis, draft angle, zebra stripe modes
- [ ] `viewport`: Add measurement tool (point-to-point distance display)
- [ ] `viewport`: Implement construction planes (user-defined work planes)
- [ ] `ui`: Implement content browser thumbnail grid with preview on hover
- [ ] `ui`: Implement node editor panel (hosts material graph editor)
- [ ] `scene`: Implement terrain component with heightmap storage

---

### Stage 4: Animation & Physics (Weeks 21-28)

**Goal:** Keyframe animation system, skeletal animation, timeline editor, physics simulation, IES lights, toon rendering.

#### Crate: `animation` [Model Layer]

**Cargo.toml:**
```toml
[package]
name = "animation"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
glam = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
indexmap = { workspace = true }
```

**TODO Checklist:**
- [ ] Define `AnimationClip` struct: name, duration, sample rate, tracks
- [ ] Define `AnimationTrack` struct: target entity + component + field, keyframes
- [ ] Define `Keyframe<T>` struct: time, value, interpolation mode (Step, Linear, Bezier), tangent handles
- [ ] Implement `CurveEvaluator`: evaluate track value at arbitrary time with interpolation
- [ ] Implement Bezier curve tangent editing (auto, free, aligned, weighted)
- [ ] Implement `AnimationPlayer`: play, pause, stop, loop, ping-pong, speed control
- [ ] Implement `TimelineViewModel`: current time, playback state, visible range, zoom
- [ ] Implement `KeyframeEditor`: add/remove/move keyframes, edit tangents in curve editor
- [ ] Implement `DopeSheetView`: simplified keyframe overview per track
- [ ] Implement `CurveEditorView`: graph editor with curves, tangent handles, grid
- [ ] Implement auto-key mode: automatically insert keyframes when transforming objects during playback
- [ ] Implement `Skeleton` struct: bone hierarchy, rest pose, bind matrices
- [ ] Implement `SkinComponent`: skeleton reference, bone weights per vertex, bone influences
- [ ] Implement skeletal animation blending: additive, override, blend by weight
- [ ] Implement animation import from glTF animations
- [ ] Implement `AnimationStateMachine`: states, transitions, conditions, blend trees
- [ ] Implement `AnimationEvent`: fire events at specific keyframe times (callbacks, sounds)
- [ ] Implement onion skinning: ghost previous/next frames during posing
- [ ] Implement animation preview in viewport with play controls
- [ ] Write unit tests for interpolation, state machine transitions, skeletal blending

**Acceptance Criteria:**
- Objects animate along keyframed transform curves
- Timeline shows and edits keyframes with dope sheet and curve views
- Skeletal animation plays imported glTF animations
- State machine transitions work with conditions
- Auto-key records transform changes as keyframes

---

#### Crate: `physics` [Model Layer]

**Cargo.toml:**
```toml
[package]
name = "physics"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
rapier3d = { workspace = true }
glam = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
```

**TODO Checklist:**
- [ ] Implement `PhysicsWorld` wrapper over `rapier3d` pipeline (gravity, timestep)
- [ ] Implement `RigidBodyComponent`: dynamic, kinematic, static, mass, linear/angular damping
- [ ] Implement `ColliderComponent`: shape (box, sphere, capsule, convex hull, trimesh), material (friction, restitution)
- [ ] Implement automatic collider generation from mesh bounding shapes
- [ ] Implement `ColliderEditor` in inspector: visualize collider shape as wireframe overlay
- [ ] Implement physics simulation step: sync transforms between ECS and Rapier
- [ ] Implement play/pause physics simulation in editor (preview mode)
- [ ] Implement `JointComponent`: fixed, revolute, prismatic, spherical joints with limits
- [ ] Implement joint visualization: render joint constraints as debug lines
- [ ] Implement `CharacterController`: basic capsule controller for play-in-editor
- [ ] Implement raycasting through Rapier for accurate physics-based picking
- [ ] Implement `ForceComponent`: constant force, impulse, torque application
- [ ] Implement collision event reporting: OnCollisionEnter, OnCollisionExit, OnTrigger
- [ ] Implement physics debug visualization: collider shapes, contacts, AABBs, forces
- [ ] Implement physics material library: presets for wood, metal, rubber, ice, etc.
- [ ] Implement gravity override per entity
- [ ] Implement physics recording: bake simulation to keyframe animation
- [ ] Write unit tests for collider creation, simulation step, joint constraints

**Acceptance Criteria:**
- Objects with rigid bodies fall under gravity and collide
- Collider shapes match mesh geometry
- Joints constrain connected bodies correctly
- Play-in-editor runs physics simulation
- Simulation can be baked to animation keyframes

---

**Additional Stage 4 Features:**

- [ ] `render`: Implement IES profile light support (photometric data loading and rendering)
- [ ] `render`: Implement toon/cel shading render style with outline pass
- [ ] `render`: Implement tube/capsule light type
- [ ] `render`: Implement emissive mesh lighting
- [ ] `scene`: Add `IESProfileComponent` with `.ies` file reference
- [ ] `ui`: Implement animation timeline panel with dope sheet and curve editor
- [ ] `ui`: Implement physics toolbar (play/pause/step simulation)
- [ ] `viewport`: Implement physics debug draw overlay

---

### Stage 5: Scripting & Plugins (Weeks 29-36)

**Goal:** Lua scripting engine, script editor, hot-reload, plugin API, plugin registry, and particle system.

#### Crate: `scripting` [ViewModel Layer]

**Cargo.toml:**
```toml
[package]
name = "scripting"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
mlua = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
notify = { workspace = true }
parking_lot = { workspace = true }
```

**TODO Checklist:**
- [ ] Initialize `mlua` VM with Luau mode and sandboxing
- [ ] Implement `ScriptComponent`: reference to `.lua` file, enabled flag, property overrides
- [ ] Expose `Entity` API to Lua: get/set transform, name, components, children
- [ ] Expose `Scene` API to Lua: find entities, spawn, destroy, query
- [ ] Expose `Input` API to Lua: is_key_pressed, is_mouse_down, mouse_position, axis values
- [ ] Expose `Time` API to Lua: delta_time, elapsed, frame_count
- [ ] Expose `Physics` API to Lua: raycast, apply_force, set_velocity
- [ ] Expose `Math` API to Lua: Vec3, Quat, Mat4 with operator overloads
- [ ] Implement script lifecycle: `on_start()`, `on_update(dt)`, `on_destroy()`, `on_collision(other)`
- [ ] Implement `ScriptHotReload`: watch `.lua` files, reload VM on change without restarting editor
- [ ] Implement `ScriptEditorPanel`: syntax-highlighted text editor with line numbers
- [ ] Implement script error handling: display errors in console panel, highlight error line
- [ ] Implement `ScriptConsole`: print() outputs to a dedicated console panel
- [ ] Implement script debugging: breakpoints, variable inspection, step-through (basic)
- [ ] Implement property binding: expose Lua variables as inspector-editable properties
- [ ] Implement script templates: starter scripts for common patterns (controller, trigger, spawner)
- [ ] Implement sandboxing: restrict file system access, limit memory, cap execution time
- [ ] Implement inter-script communication: global event bus accessible from Lua
- [ ] Write integration tests for Lua API bindings, hot-reload, sandboxing

**Acceptance Criteria:**
- Lua scripts attach to entities and run during play-in-editor
- Scripts can move objects, respond to input, and interact with physics
- Hot-reload updates script behavior without restarting
- Script errors display in console without crashing editor
- Script properties are editable in the inspector

---

#### Crate: `plugins` [Service Layer]

**Cargo.toml:**
```toml
[package]
name = "plugins"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scripting = { path = "../scripting" }
serde = { workspace = true }
toml = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
walkdir = { workspace = true }
parking_lot = { workspace = true }
uuid = { workspace = true }
```

**TODO Checklist:**
- [ ] Define `PluginManifest` struct: name, version, author, description, entry point, dependencies
- [ ] Define `Plugin` trait: `on_load()`, `on_unload()`, `on_update()`, `register_commands()`, `register_panels()`
- [ ] Implement `PluginRegistry`: discover plugins from `plugins/` directory
- [ ] Implement `PluginLoader`: load plugin manifest, validate, initialize
- [ ] Implement `PluginSandbox`: restrict plugin access to safe API surface
- [ ] Implement `PluginAPI` facade: safe accessors for scene, assets, UI, commands
- [ ] Implement plugin hot-reload: watch plugin directory, reload on change
- [ ] Implement plugin dependency resolution: topological sort of plugin load order
- [ ] Implement plugin settings UI: auto-generate settings panel from manifest schema
- [ ] Implement plugin enable/disable toggle without unloading
- [ ] Implement `PluginCommand` registration: plugins can add commands to the command palette
- [ ] Implement `PluginPanel` registration: plugins can add custom egui panels
- [ ] Implement `PluginMenu` registration: plugins can add menu items
- [ ] Implement plugin marketplace data model (local only, no network in MVP)
- [ ] Implement built-in plugin: "FBX Importer" as reference implementation
- [ ] Write tests for manifest parsing, load/unload lifecycle, sandboxing

**Acceptance Criteria:**
- Plugins load from directory and execute lifecycle hooks
- Plugin commands appear in command palette
- Plugin panels render in the UI
- Hot-reload updates plugin behavior
- Sandboxing prevents file system abuse

---

**Additional Stage 5 Features:**

- [ ] `render`: Implement particle system renderer (billboard quads, GPU instanced)
- [ ] `scene`: Define `ParticleSystemComponent`: emitter shape, rate, lifetime, velocity, color over life, size over life
- [ ] `ui`: Implement particle system editor panel
- [ ] `ui`: Implement script editor panel with syntax highlighting
- [ ] `ui`: Implement plugin manager panel (list, enable/disable, settings)
- [ ] `materials`: Add particle material support (additive, alpha blended)

---

### Stage 6: Polish & Integration (Weeks 37-44)

**Goal:** Project management, VCS integration, profiler, post-processing, export pipeline, command palette, multi-viewport, crash recovery.

#### Crate: `project` [Service Layer]

**Cargo.toml:**
```toml
[package]
name = "project"
version.workspace = true
edition.workspace = true

[dependencies]
core = { path = "../core" }
scene = { path = "../scene" }
assets = { path = "../assets" }
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }
uuid = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }
thiserror = { workspace = true }
walkdir = { workspace = true }
tokio = { workspace = true }
```

**TODO Checklist:**
- [ ] Define `ProjectManifest` struct: name, version, author, engine version, scenes list, build targets
- [ ] Implement `create_project(name, path)`: scaffold directory structure, write manifest
- [ ] Implement `open_project(path)`: validate manifest, load asset registry, open last scene
- [ ] Implement `save_project()`: write manifest, save all dirty scenes and assets
- [ ] Implement recent projects list with thumbnails (stored in user config)
- [ ] Implement `ProjectSettings` panel: project name, default scene, build configuration
- [ ] Implement VCS integration: detect `.git`, show file status (modified/added/untracked)
- [ ] Implement VCS diff viewer: show changes since last commit in a side panel
- [ ] Implement auto-save: configurable interval (default 5 min), save to `.autosave/` directory
- [ ] Implement crash recovery: detect unclean shutdown, offer to restore from autosave
- [ ] Implement `ExportPipeline`: export scene to various formats
- [ ] Implement glTF/GLB export: serialize scene to standard glTF 2.0
- [ ] Implement FBX export (via intermediary or plugin)
- [ ] Implement OBJ export for simple mesh exchange
- [ ] Implement texture baking: bake material node graphs to texture maps
- [ ] Implement build system: package game assets into optimized runtime format

**Acceptance Criteria:**
- Projects create, open, save with full asset integrity
- VCS status shows in file browser / outliner
- Auto-save and crash recovery work reliably
- glTF export produces valid files loadable in other tools
- Build system packages all assets

---

**Additional Stage 6 Features (implemented across crates):**

- [ ] `render`: Implement post-processing stack: Bloom, Tone Mapping (ACES), Color Grading, Vignette, FXAA/TAA, SSAO, Depth of Field, Motion Blur
- [ ] `render`: Implement progressive path tracer for reference renders
- [ ] `render`: Implement volumetric fog / god rays
- [ ] `render`: Implement PBR Preview mode with full IBL pipeline
- [ ] `render`: Implement portal light type
- [ ] `ui`: Implement command palette (Ctrl+P): fuzzy search all commands, recent, categories
- [ ] `ui`: Implement multi-viewport layout (2x2, side-by-side, custom splits)
- [ ] `ui`: Implement profiler panel: frame time graph, draw call count, GPU time, memory breakdown
- [ ] `ui`: Implement console panel: log output, script print, command history, filter by severity
- [ ] `ui`: Implement preferences dialog: keybindings, theme, auto-save interval, units, grid settings
- [ ] `ui`: Implement welcome screen: recent projects, new project wizard, template gallery
- [ ] `viewport`: Implement multi-viewport camera sync / independent controls
- [ ] `viewport`: Implement screen-space annotations and sticky notes
- [ ] `core`: Implement user preferences save/load to OS-appropriate config directory
- [ ] `core`: Implement undo history panel showing command descriptions with icons
- [ ] `core`: Implement localization framework (i18n) with English as base language
- [ ] `app`: Implement window state save/restore (position, size, maximized, monitor)
- [ ] `app`: Implement session restore: reopen panels, scenes, and tool states from last session
- [ ] `app`: Implement drag-and-drop files onto window to import assets
- [ ] `app`: Implement custom window title showing project name and current scene

---

### Summary Timeline Table

| Stage | Weeks   | Focus                    | Crates Introduced                          | Key Deliverables                                   |
|-------|---------|--------------------------|--------------------------------------------|-----------------------------------------------------|
| 1     | 1-6     | Foundation MVP           | `core`, `render`, `viewport`, `ui`, `app`  | Window, viewport, grid, camera, egui chrome, commands|
| 2     | 7-12    | Scene & Assets           | `scene`, `assets`, `inspector`             | Scene graph, .glb loading, content browser, inspector|
| 3     | 13-20   | Editing Tools            | `modeling`, `materials`                    | Mesh ops, sketch mode, material nodes, terrain       |
| 4     | 21-28   | Animation & Physics      | `animation`, `physics`                     | Keyframes, timeline, skeletal anim, rigid bodies     |
| 5     | 29-36   | Scripting & Plugins      | `scripting`, `plugins`                     | Lua scripts, hot-reload, plugin host, particles      |
| 6     | 37-44   | Polish & Integration     | `project`                                  | VCS, profiler, post-process, export, command palette |

### Crate Dependency Graph (Detailed)

```
Legend: ──► = depends on

                    ┌────────────────────────────────────┐
                    │              app                    │
                    │  (binary: integrates everything)   │
                    └──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┘
                       │  │  │  │  │  │  │  │  │  │  │
          ┌────────────┘  │  │  │  │  │  │  │  │  │  └──────────┐
          │     ┌─────────┘  │  │  │  │  │  │  │  └─────┐       │
          │     │    ┌───────┘  │  │  │  │  │  └──┐     │       │
          ▼     ▼    ▼          ▼  │  ▼  │  ▼     ▼     ▼       ▼
       ┌────┐┌──────┐┌─────────┐  │┌───┐│┌────┐┌─────┐┌──────┐┌───────┐
       │ ui ││viewpt││inspector│  ││mod││││anim││physc││script││plugins│
       └─┬──┘└──┬───┘└───┬─────┘  │└─┬─┘││└──┬─┘└──┬──┘└──┬───┘└──┬────┘
         │      │        │        │   │  ││   │     │      │       │
         │      │        │   ┌────┘   │  ││   │     │      │       │
         │      │        │   │        │  ││   │     │      │       │
         │      ▼        │   ▼        │  ▼│   │     │      │       │
         │  ┌───────┐    │ ┌─────┐    │┌──┴┐  │     │      │       │
         │  │render │    │ │mater│    ││prjt│  │     │      │       │
         │  └───┬───┘    │ └──┬──┘    │└─┬──┘  │     │      │       │
         │      │        │    │       │  │     │     │      │       │
         ├──────┤   ┌────┤    │       │  │     │     │      │       │
         │      │   │    │    │       │  │     │     │      │       │
         │      ▼   │    ▼    │       │  │     │     │      │       │
         │  ┌──────┐│┌──────┐ │       │  │     │     │      │       │
         │  │assets│││scene │◄┼───────┼──┼─────┼─────┼──────┘       │
         │  └──┬───┘│└──┬───┘ │       │  │     │     │              │
         │     │    │   │     │       │  │     │     │              │
         │     ▼    │   ▼     │       │  │     │     │              │
         │  ┌──────────────┐  │       │  │     │     │              │
         └─►│     core     │◄─┴───────┴──┴─────┴─────┴──────────────┘
            └──────────────┘
```

---

## Part 4: Command System (Rhino-inspired)

### Overview

The command line is a first-class interface element in Forge, positioned at the bottom of the window. It accepts typed commands, provides auto-complete, and supports parametric input.

### Command Syntax

Commands follow a `command [args...] [flags]` pattern:

```
> box
   Creates a unit box at origin. Prompts for corner points interactively.

> box 2 3 4
   Creates a box with dimensions 2 x 3 x 4 at origin.

> box 2 3 4 --at 1 0 0
   Creates a 2x3x4 box centered at position (1, 0, 0).

> sphere --radius 2.5 --segments 64
   Creates a sphere with radius 2.5 and 64 segments.

> extrude --distance 5
   Extrudes selected faces by 5 units along their normals.

> extrude --direction 0 1 0 --distance 3
   Extrudes selected faces 3 units along the Y axis.

> move 0 0 5
   Translates selected objects by (0, 0, 5).

> rotate --axis 0 1 0 --angle 45
   Rotates selection 45 degrees around the Y axis.

> copy --offset 3 0 0
   Copies selection offset by (3, 0, 0).

> mirror --plane xz
   Mirrors selection across the XZ plane.

> fillet --radius 0.2
   Fillets selected edges with 0.2 unit radius.

> array --count 5 --spacing 2 0 0
   Creates a linear array of 5 copies spaced 2 units apart on X.

> select --name "Cube*"
   Selects all entities matching the glob pattern "Cube*".

> layer --new "Electrical" --color #ff9f43
   Creates a new layer named "Electrical" with orange color.

> save
   Saves the current project.

> undo
   Undoes the last command.

> measure
   Enters measurement mode for point-to-point distance.

> snap --toggle mid
   Toggles the midpoint snap on/off.

> render --style wireframe
   Switches viewport render style to wireframe.

> view --front
   Snaps camera to front orthographic view.

> export --format glb --path "./export/scene.glb"
   Exports the scene as a GLB file.
```

### Auto-Complete

- Typing triggers a dropdown of matching commands
- Tab completes the current best match
- Arrow keys navigate suggestions
- Suggestions show: command name, brief description, keyboard shortcut (if bound)
- Fuzzy matching: `ext` matches `extrude`, `export`, `extend`
- Recent commands appear at the top of suggestions
- Context-aware: only shows applicable commands (e.g., mesh commands only when a mesh is selected)

### Command History

- Up/Down arrow keys cycle through previously executed commands
- History persists across sessions (saved in user config)
- `history` command prints last N commands
- `history --clear` clears command history
- `!N` re-executes the Nth command from history
- `!!` re-executes the last command

### Parametric Input

Commands that require geometric input support interactive and inline modes:

**Interactive mode (no args):**
```
> box
  Pick first corner: (click in viewport)
  Pick opposite corner: (click in viewport)
  Enter height: 3 (type and press Enter)
  Box created: 4.2 x 2.8 x 3.0
```

**Inline mode (with args):**
```
> box 2 3 4
  Box created: 2.0 x 3.0 x 4.0
```

**Mixed mode:**
```
> extrude
  Select faces: (click faces in viewport, Enter to confirm)
  Enter distance: 2.5
  Extrude complete: 4 faces extruded by 2.5
```

### Alias System

Users can define short aliases for frequently used commands:

```toml
# aliases.toml
[aliases]
b   = "box"
s   = "sphere"
ext = "extrude"
mv  = "move"
rot = "rotate"
cp  = "copy"
del = "delete"
fa  = "select --all"
fn  = "select --none"
fi  = "select --invert"
wf  = "render --style wireframe"
sd  = "render --style solid"
```

**Built-in alias management commands:**
```
> alias b box           # Create alias
> alias --list          # List all aliases
> alias --remove b      # Remove alias
> alias --reset         # Restore default aliases
```

### Command Registration

Commands are registered via the `Command` trait in the `core` crate:

```rust
pub trait CommandDescriptor {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn category(&self) -> &str;
    fn args(&self) -> &[ArgDescriptor];
    fn execute(&self, ctx: &mut CommandContext, args: &ParsedArgs) -> Result<CommandResult>;
}

pub struct ArgDescriptor {
    pub name: String,
    pub arg_type: ArgType,
    pub required: bool,
    pub default: Option<String>,
    pub description: String,
}

pub enum ArgType {
    Float,
    Int,
    Vec3,
    String,
    Bool,
    Enum(Vec<String>),
    EntityRef,
}
```

---

## Part 5: Object Snap System (Rhino-inspired)

### Overview

Object snaps (Osnaps) constrain the cursor to specific geometric features, enabling precise modeling without typing exact coordinates. Snaps are toggled individually via the snap toolbar or command line.

### Snap Types

| Snap Type       | Icon  | Description                                           | Detection Logic                              |
|-----------------|-------|-------------------------------------------------------|----------------------------------------------|
| **End**         | `[E]` | Endpoints of edges, curves, lines                     | Find nearest vertex/endpoint within radius   |
| **Mid**         | `[M]` | Midpoint of edges and curves                          | Compute midpoint of nearest edge             |
| **Center**      | `[C]` | Center of circles, arcs, spheres, polygons            | Find centroid of circular/polygon features    |
| **Near**        | `[N]` | Nearest point on any curve or surface                 | Project cursor ray onto nearest geometry      |
| **Perpendicular** | `[P]` | Point perpendicular to a line from the last point  | Compute foot of perpendicular from reference  |
| **Tangent**     | `[T]` | Point of tangency on circles and curves               | Find tangent point from last point to curve   |
| **Quadrant**    | `[Q]` | 0, 90, 180, 270 degree points on circles/arcs         | Compute quadrant points of circular features  |
| **Intersection**| `[I]` | Intersection of two curves or edges                   | Compute edge-edge intersection in 3D         |
| **Knot**        | `[K]` | Knot points on splines and NURBS curves               | Find nearest knot parameter on curve          |
| **Grid**        | `[G]` | Nearest grid point on the construction plane           | Round cursor position to grid interval        |
| **Face**        | `[F]` | Center of a mesh face                                 | Compute face centroid of nearest face         |
| **Vertex**      | `[V]` | Nearest mesh vertex                                   | Find nearest vertex within snap radius        |
| **On Surface**  | `[O]` | Constrain to any surface                              | Project cursor onto nearest surface           |

### Snap Behavior

- **Snap radius**: 15 pixels (configurable). The cursor snaps to the nearest snap point within this radius.
- **Priority**: When multiple snaps are within radius, priority is: active snap types in toolbar order (left to right).
- **Visual indicator**: A colored dot with label appears at the snap point. Dot color matches the snap type. A tooltip shows the snap type name and coordinates.
- **One-shot snap**: Hold a modifier key (e.g., Shift) and click a snap type to use it for the next pick only, without toggling it permanently.
- **Snap override**: Type a snap prefix during interactive input to override (e.g., `end` then click).
- **Disable all**: Press the snap toggle button or use `snap --off` to disable all snaps temporarily.

### Snap Toolbar Layout

```
[Snap: ON] | [E] [M] [C] [N] [P] [T] [Q] [I] [K] [G] [F] [V] [O]
```

Each button toggles the corresponding snap type. Active snaps are highlighted with `accent-primary`.

### Snap Detection Algorithm

1. Cast a ray from the camera through the mouse cursor position.
2. For each active snap type, compute candidate snap points within the snap radius (in screen space).
3. Rank candidates by distance from cursor (screen space) and snap type priority.
4. Select the highest-priority candidate within radius.
5. Display snap indicator at the winning snap point.
6. If the user clicks, the snap point position is used instead of the raw cursor position.

### Snap Configuration

```toml
# snap.config.toml
[snap]
enabled = true
radius_px = 15
grid_size = 1.0
grid_subdivisions = 10
show_indicator = true
indicator_size = 8.0
show_tooltip = true
show_crosshair_at_snap = true

[snap.active]
end = true
mid = true
center = true
near = false
perpendicular = false
tangent = false
quadrant = false
intersection = true
knot = false
grid = true
face = false
vertex = true
on_surface = false
```

---

## Part 6: Node Editor (Grasshopper-inspired)

### Overview

The node editor is a visual programming environment for creating material graphs, procedural geometry, animation state machines, and particle systems. Inspired by Grasshopper's clean, data-flow-driven interface, it uses the Crystalline theme with color-coded pins and wires.

### Visual Style

- **Background**: `#131316` with a 20px dot grid in `#1f1f2140`
- **Nodes**: Rounded rectangles with `#1f1f21e6` body and `#262629` header
- **Node border**: `#2e2e32`, `#4eff93` when selected
- **Header**: Displays node name (Inter Medium 11px) and a small icon
- **Pins**: Colored circles (6px) on node edges, color indicates data type
- **Wires**: Bezier curves between pins, colored by data type, 2px thick
- **Mini-preview**: Small output preview rendered on applicable nodes
- **Pan**: MMB drag or RMB drag on background
- **Zoom**: Scroll wheel
- **Selection**: Click node, box select, Shift+click for multi-select
- **Node creation**: Right-click background opens categorized creation menu

### Pin / Wire Color Coding

| Data Type   | Pin Color   | Hex       |
|-------------|-------------|-----------|
| Float       | Cyan-Green  | `#4eff93` |
| Vec2        | Light Green | `#93ff4e` |
| Vec3        | Blue        | `#3e55ff` |
| Vec4        | Purple      | `#9f3eff` |
| Color (RGB) | Red-Pink    | `#e94560` |
| Bool        | Orange      | `#ff9f43` |
| Texture     | Gray        | `#9b9ba1` |
| Mesh        | White       | `#e7e7ea` |
| Any         | Yellow      | `#ffe74e` |

### Node Categories

#### Material Graph Nodes

| Category      | Nodes                                                                  |
|---------------|------------------------------------------------------------------------|
| Output        | PBR Output, Unlit Output, Custom Output                               |
| Textures      | Texture Sample, Render Target, Gradient Map                           |
| Math          | Add, Subtract, Multiply, Divide, Power, Sqrt, Abs, Negate, Fract     |
| Trigonometry  | Sin, Cos, Tan, Asin, Acos, Atan2                                     |
| Interpolation | Lerp, Smoothstep, Step, Clamp, Remap, InverseLerp                    |
| Vector        | Split, Combine, Normalize, Dot, Cross, Length, Reflect, Transform     |
| Color         | HSV to RGB, RGB to HSV, Brightness, Contrast, Saturation, Invert, Overlay |
| Procedural    | Perlin Noise, Simplex Noise, Worley Noise, Checker, Brick, Voronoi   |
| UV            | Tiling, Offset, Rotate, Projection, Parallax, Triplanar              |
| Constants     | Float, Vec2, Vec3, Vec4, Color, Texture                              |
| Utility       | Time, Screen Position, View Direction, Camera Distance, Fresnel      |

#### Procedural Geometry Nodes

| Category      | Nodes                                                                  |
|---------------|------------------------------------------------------------------------|
| Primitives    | Box, Sphere, Cylinder, Cone, Torus, Plane, Grid                       |
| Transform     | Translate, Rotate, Scale, Mirror, Bend, Twist, Taper                  |
| Mesh Ops      | Subdivide, Decimate, Smooth, Boolean, Merge, Separate                 |
| Curves        | Line, Arc, Circle, Bezier, NURBS, Helix, Loft Surface                 |
| Arrays        | Linear Array, Radial Array, Grid Array, Scatter                       |
| Deformers     | Noise Displace, Lattice, Wrap, Shrinkwrap                             |
| Analysis      | Normals, Curvature, Area, Volume, Bounds                              |
| Input/Output  | Geometry Input, Geometry Output, Attribute Get/Set                    |

#### Animation State Machine Nodes

| Category      | Nodes                                                                  |
|---------------|------------------------------------------------------------------------|
| States        | Animation State, Entry State, Exit State, Any State                   |
| Transitions   | Transition, Condition, Blend Transition, Sync Transition              |
| Blending      | Blend Tree, 1D Blend, 2D Blend, Additive, Override                   |
| Parameters    | Float Param, Bool Param, Trigger Param, Int Param                     |
| Events        | Animation Event, On Enter, On Exit, On Update                         |
| Timing        | Speed Multiplier, Time Remap, Sync Group                              |

#### Particle System Nodes

| Category      | Nodes                                                                  |
|---------------|------------------------------------------------------------------------|
| Emitters      | Point, Box, Sphere, Mesh Surface, Edge, Burst                         |
| Initializers  | Lifetime, Speed, Direction, Size, Color, Rotation                     |
| Modifiers     | Gravity, Wind, Turbulence, Drag, Attraction, Vortex                   |
| Over Lifetime | Color Over Life, Size Over Life, Speed Over Life, Rotation Over Life   |
| Collision     | Plane Collision, World Collision, Kill, Bounce                         |
| Rendering     | Billboard, Mesh, Trail, Ribbon, Sprite Sheet                          |
| Sub-Systems   | Spawn on Event, Inherit Velocity, Sub-Emitter                         |

### Node Editor Interaction

**Creating nodes:**
1. Right-click on canvas background
2. Category menu appears (Material, Geometry, Animation, Particles)
3. Navigate sub-categories to find desired node
4. Click to place node at cursor position
5. Alternatively: type node name in search bar at top of menu

**Connecting pins:**
1. Click and drag from an output pin
2. A wire follows the cursor
3. Hover over compatible input pin (highlighted green)
4. Release to connect
5. Incompatible pins are dimmed and show a red indicator
6. Dropping on empty space opens a filtered creation menu showing only nodes with compatible inputs

**Wire routing:**
- Wires use cubic Bezier curves with automatic tangent computation
- Horizontal tangent strength proportional to horizontal distance between pins
- Wires avoid overlapping node bodies where possible
- Wire color matches the data type of the output pin
- Active/selected wires increase thickness to 3px and brighten

**Node groups:**
- Select multiple nodes, right-click, "Group Nodes"
- Groups have a colored background rectangle with a title
- Groups can be collapsed to a single node showing group inputs/outputs
- Useful for organizing complex graphs

**Bookmarks and navigation:**
- Ctrl+number to set a bookmark at current pan/zoom position
- Number key to jump to a bookmark
- Minimap in corner shows overview of entire graph with viewport indicator

### Node Editor Data Model

```rust
pub struct NodeGraph {
    pub id: Uuid,
    pub name: String,
    pub graph_type: GraphType,
    pub nodes: IndexMap<NodeId, Node>,
    pub wires: Vec<Wire>,
    pub groups: Vec<NodeGroup>,
    pub metadata: GraphMetadata,
}

pub enum GraphType {
    Material,
    ProceduralGeometry,
    AnimationStateMachine,
    ParticleSystem,
}

pub struct Node {
    pub id: NodeId,
    pub type_name: String,
    pub position: Vec2,
    pub inputs: Vec<Pin>,
    pub outputs: Vec<Pin>,
    pub properties: IndexMap<String, PropertyValue>,
    pub collapsed: bool,
    pub preview_enabled: bool,
}

pub struct Pin {
    pub id: PinId,
    pub name: String,
    pub data_type: PinDataType,
    pub direction: PinDirection,
    pub default_value: Option<PropertyValue>,
}

pub struct Wire {
    pub id: WireId,
    pub from_node: NodeId,
    pub from_pin: PinId,
    pub to_node: NodeId,
    pub to_pin: PinId,
}

pub struct NodeGroup {
    pub name: String,
    pub color: ColorRGBA,
    pub node_ids: Vec<NodeId>,
    pub collapsed: bool,
}
```

---

*End of Forge Editor Application Roadmap*
