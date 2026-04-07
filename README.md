# GameCAD

A modular, Crystalline-themed CAD/Game Editor built in pure Rust with wgpu + egui.

Combines the precision of **Rhino 3D**, the parametric power of **Fusion 360**, and the game-ready workflow of **Unreal Engine** — all in a single native desktop application.

## Features

- **Fusion 360-style tabbed workspace** — Map Editor, Gameplay, Object Editor, Script Editor, Material Editor, Animation, Physics
- **Crystalline dark/light theme** — inspired by Baby Audio's Crystalline plugin, with 8-band viewport gradients
- **8 render modes** — Shaded, Wireframe, Shaded+Wire, Unlit, Ghost/X-Ray, Normals, Depth, Clay
- **Unreal Engine mouse controls** — right-drag orbit, middle-drag pan, scroll zoom, Alt+drag Maya-style
- **Nested context menus** — Add meshes, lights, cameras; transform, snap, view presets
- **Move/Rotate/Scale tools** — with visual gizmo indicators and grid snapping
- **Material node graph editor** — Grasshopper-inspired visual programming with WGSL shader compilation
- **Half-edge mesh modeling** — extrude, bevel, CSG (union/subtract/intersect), Catmull-Clark subdivision
- **Scene graph** — hierarchical nodes with RON serialization, layer management
- **Asset database** — import pipeline, thumbnails, UUID-based references
- **Agent progress panel** — live task tracking with animated progress bars
- **Command palette** — Ctrl+Shift+P fuzzy search for all commands

## Architecture

- **MVVM pattern** — Model (core, scene, assets) / ViewModel (viewport, inspector) / View (ui, app)
- **14 modular crates** — each feature is a separate library with clean APIs
- **Pure Rust** — no JavaScript, no web stack, fully native multiplatform
- **112 source files** | **16,600+ lines** | **172 tests**

## Crates

| Crate | Purpose |
|-------|---------|
| `core` | Math, ECS, EventBus, Undo/Redo, IDs, Clock |
| `render` | wgpu pipeline, 6 WGSL shaders, .glb loader, PBR |
| `ui` | Crystalline theme engine, Fusion tabs, toolbar, panels |
| `viewport` | OrbitCamera, gizmo, selection, grid, ray picking |
| `scene` | Scene graph, nodes, RON serialization, layers |
| `assets` | Asset database, importer, thumbnails, metadata |
| `inspector` | Property widgets, node inspector |
| `modeling` | Half-edge mesh, primitives, operations, CSG |
| `materials` | PBR materials, node graph, WGSL compiler |

## Quick Start

```bash
# Clone
git clone https://github.com/YOUR_USERNAME/GameCAD.git
cd GameCAD

# Run
cargo run -p forge-editor-app

# Test
cargo test --workspace
```

## Controls

| Input | Action |
|-------|--------|
| Right-drag | Orbit camera |
| Middle-drag | Pan camera |
| Scroll | Zoom |
| Alt+Left-drag | Maya-style orbit |
| Left-click | Select entity |
| Ctrl+click | Toggle selection |
| Right-click | Context menu |
| W/E/R | Move/Rotate/Scale tools |
| Z | Cycle render styles |
| G | Toggle grid |
| Ctrl+T | Toggle dark/light theme |
| Ctrl+Shift+P | Command palette |
| F | Focus on selection |

## Requirements

- Rust 1.85+ (edition 2024)
- GPU with Vulkan, Metal, or DX12 support

## License

MIT OR Apache-2.0
