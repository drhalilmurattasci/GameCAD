# Forge Editor -- Measurement, Units & Grid System Design

**Status:** Brainstorm / Design Draft
**Scope:** Internal architecture, display pipeline, grid, snapping, measurement tools, chunk system, precision strategy
**Audience:** Engine developers, editor UI developers, future contributors

---

## Table of Contents

1. [The Problem](#part-1-the-problem)
2. [Internal vs Display Units](#part-2-internal-vs-display-units)
3. [Unit Profiles / Presets](#part-3-unit-profiles--presets)
4. [Chunk System](#part-4-chunk-system)
5. [Ruler / Measurement Tools](#part-5-ruler--measurement-tools)
6. [Grid System](#part-6-grid-system)
7. [Precision & Floating Point](#part-7-precision--floating-point)
8. [Scale Reference Objects](#part-8-scale-reference-objects)
9. [Unit-Aware UI Patterns](#part-9-unit-aware-ui-patterns)
10. [Configuration File Format](#part-10-configuration-file-format)
11. [Cross-Reference Table](#part-11-cross-reference-table)
12. [Implementation Priority](#part-12-implementation-priority)
13. [Open Questions & Future Work](#part-13-open-questions--future-work)
14. [Coordinate System Modes](#part-14-coordinate-system-modes)
15. [Angular Units](#part-15-angular-units)
16. [Time Units for Animation](#part-16-time-units-for-animation)
17. [Velocity & Physics Units](#part-17-velocity--physics-units)
18. [Color Units & Spaces](#part-18-color-units--spaces)
19. [Area & Volume Display](#part-19-area--volume-display)
20. [DPI / Pixels Per Unit (2D Mode)](#part-20-dpi--pixels-per-unit-2d-mode)
21. [Export Unit Mapping](#part-21-export-unit-mapping)
22. [Measurement History & Bookmarks](#part-22-measurement-history--bookmarks)
23. [Smart Unit Detection](#part-23-smart-unit-detection)
24. [Construction Plane System (Rhino-inspired)](#part-24-construction-plane-system-rhino-inspired)

---

## Part 1: The Problem

Game engines and CAD tools use wildly different unit systems. There is no industry standard. Every tool picks its own internal unit, its own display convention, and its own grid/snap defaults. Users moving between tools -- or building content for a specific target engine -- must constantly perform mental conversions.

### 1.1 Survey of Existing Conventions

| Tool / Domain | Unit Convention | Notes |
|---|---|---|
| Minecraft | 1 block = 1 meter, chunk = 16x16x256 blocks | Voxel grid, integer coordinates |
| Unreal Engine | 1 unit = 1 cm (100 units = 1 meter) | Historical, deeply embedded in tooling |
| Unity | 1 unit = 1 meter | De facto standard for indie/mobile |
| Godot | 1 unit = 1 meter (2D: 1 unit = 1 pixel) | Dual convention between 2D and 3D |
| Blender | 1 unit = 1 meter (configurable) | Scene-level setting, rarely changed |
| AutoCAD / Fusion 360 | mm, cm, m, inches, feet (user-configurable) | Per-document unit, full imperial support |
| Rhino 3D | Configurable (mm default for industrial, meters for architecture) | Template-driven |
| 3ds Max | System units + display units (separate concepts) | Two-layer system, source of confusion |
| SketchUp | Inches (US) or mm (metric) | Region-dependent default |
| DCS World / Flight Sims | Nautical miles, feet (altitude), knots | Mixed units within same context |
| Golf games | Yards, feet, inches | Compound display common |
| Architectural (US) | Feet + inches (imperial) | Compound: 5' 4 3/8" |
| Architectural (Metric) | Meters or millimeters | Varies by country |
| Engineering / CNC | mm or thousandths of an inch (mils/thou) | Sub-millimeter precision required |
| Jewelry / Watch | mm with 0.01mm precision | Very small scale, high precision |
| Urban planning | km, hectares | Very large scale, low precision per-point |
| Terrain / GIS | Meters with UTM coordinates, lat/long | Mixed coordinate systems, earth curvature |
| Semiconductor | Nanometers, micrometers | Extreme precision at microscopic scale |
| Naval / Maritime | Nautical miles, fathoms, knots | Historical units still in active use |
| Space / Orbital | Kilometers, astronomical units | Extreme scale range |

### 1.2 Core Tensions

These conventions create several fundamental tensions the Forge Editor must resolve:

1. **Scale range**: A single project might contain objects from millimeter detail (a screw thread) to kilometer extent (terrain). The unit system must not collapse precision at either extreme.

2. **Cultural expectation**: A US architect expects feet-and-inches. A German mechanical engineer expects millimeters. A game developer expects meters. The editor must feel native to each.

3. **Target engine compatibility**: Content authored in Forge may target Unreal (cm), Unity (m), Godot (m), or a custom engine. Export must produce correct scale without manual fixup.

4. **Compound units**: Imperial architecture uses feet-and-inches with fractional inches (5' 4 3/16"). This is not a simple scalar -- it requires special parsing, display formatting, and rounding logic.

5. **Mixed units in one scene**: A flight simulator scene might show altitude in feet, distance in nautical miles, and runway dimensions in meters. The system must support per-axis or per-context unit overrides.

6. **Grid/snap coherence**: The grid must snap to "nice" numbers in the display unit, not in meters. Snapping to 0.0254m (1 inch) is correct when the user works in inches, but the grid must show clean inch markings.

### 1.3 Design Goal

The Forge Editor needs a universal measurement system that:

- Works transparently for any domain (games, CAD, architecture, terrain, etc.)
- Stores everything internally in one canonical unit (meters)
- Converts to/from any display unit at the UI boundary
- Provides sensible presets so users are productive immediately
- Allows full customization for edge cases
- Handles precision across massive scale ranges (nm to km)
- Supports compound unit display (feet+inches, degrees+minutes+seconds)
- Integrates with grid, snap, ruler, and measurement tools coherently

---

## Part 2: Internal vs Display Units

### 2.1 The Two-Layer Architecture

The unit system has exactly two layers:

```
Layer 1: Internal (Engine)        Layer 2: Display (UI)
-------------------------------   -------------------------------
Always meters (f64)               User's chosen unit
Used for all math, physics,       Used for all text display,
rendering, serialization,         input parsing, grid labels,
spatial indexing, collision        ruler ticks, inspector values

Conversion happens at the boundary between these two layers.
```

This is the same pattern used by 3ds Max (system units vs display units), but we simplify by fixing the internal unit to meters rather than making it configurable. A configurable internal unit creates a combinatorial explosion of edge cases and conversion bugs. Meters are the SI base unit for length and the most common choice across game engines.

### 2.2 Internal Unit: 1 Unit = 1 Meter

- **All** engine math operates in meters. Positions, dimensions, distances, velocities, forces -- everything.
- Physics constants are in SI units (gravity = 9.81 m/s^2, etc.).
- Serialized files store meters. A saved position of (3.5, 0.0, -12.7) means 3.5 meters east, ground level, 12.7 meters south.
- The internal unit is **never** exposed to the user. They never type "meters" unless their display unit happens to be meters.
- Shader uniforms, GPU buffers, and wgpu vertex data are in meters (or camera-relative meters; see Part 7).

### 2.3 Display Unit: User's Choice

The display unit is what the user sees and types. When the display unit is millimeters:
- A 1-meter cube shows dimensions as "1000.000 mm" in the inspector.
- Typing "500" in a dimension field means 500mm = 0.5m internally.
- The grid shows mm markings.
- The ruler shows mm ticks.
- The status bar shows cursor position in mm.

### 2.4 Conversion Model

Every display unit is defined by a single conversion factor: **how many meters one display unit equals**.

| Unit | Abbreviation | Meters per Unit | Category | Primary Use Cases |
|---|---|---|---|---|
| Nanometer | nm | 1e-9 | Micro | Semiconductor, MEMS, nano-fabrication |
| Micrometer | um | 1e-6 | Micro | Precision machining, thin films, optics |
| Millimeter | mm | 0.001 | Metric | CAD, mechanical engineering, jewelry, watch |
| Centimeter | cm | 0.01 | Metric | General purpose, Unreal Engine compatibility |
| Decimeter | dm | 0.1 | Metric | Rarely used, included for completeness |
| Meter | m | 1.0 | Metric | Game engines (Unity, Godot), architecture |
| Kilometer | km | 1000.0 | Metric | Terrain, GIS, maps, urban planning |
| Thousandth of inch | thou / mil | 0.0000254 | Imperial | CNC, precision machining, PCB |
| Inch | in / " | 0.0254 | Imperial | US architecture, SketchUp, general |
| Foot | ft / ' | 0.3048 | Imperial | Architecture, golf, aviation altitude |
| Yard | yd | 0.9144 | Imperial | Golf, sports fields, landscaping |
| Mile | mi | 1609.344 | Imperial | Road distances, large-scale mapping |
| Nautical mile | nmi | 1852.0 | Aviation/Naval | Flight sims, naval, maritime |
| Fathom | fath | 1.8288 | Naval | Water depth (historical, niche) |
| Minecraft block | block | 1.0 | Game-specific | Minecraft-style voxel games |
| Unreal unit | uu | 0.01 | Game-specific | Unreal Engine compatibility |
| Roblox stud | stud | 0.28 | Game-specific | Roblox compatibility |
| Pixel | px | (see below) | 2D | 2D games, UI layout |

**Pixel note**: The pixel unit is special. Its meter-equivalent depends on a user-defined "pixels per meter" setting (default: 100 px/m for Godot-style 2D, or 16 px/m for pixel art). This setting is part of the unit profile.

**Conversion formulas**:
- Internal (meters) = Display value * meters_per_unit
- Display value = Internal (meters) / meters_per_unit

These are simple multiplications. No offsets, no nonlinear transforms. This keeps the conversion layer trivially correct.

### 2.5 Compound Unit Display

Some domains use compound units that combine two or more base units:

| Compound Format | Example | Domain |
|---|---|---|
| Feet + inches | 5' 4 3/16" | US architecture |
| Feet + decimal inches | 5' 4.1875" | US engineering |
| Degrees + minutes + seconds | 45 30' 15" | GIS, surveying, navigation |
| Meters + centimeters | 1m 75cm | Informal metric (height) |

**Feet + inches rules**:
- Display: Show feet and remaining inches. If the profile specifies fractional precision, show fractions (1/2, 1/4, 1/8, 1/16, 1/32, 1/64).
- Example at 1/16" precision: 1.6256 meters = 5' 4 1/16"
- The fractional denominator is a profile setting (power of 2, from 1 to 64).
- When the value is an exact number of feet, omit the inch portion: "6'" not "6' 0"".
- When the value is less than one foot, show inches only: 8 3/4" not 0' 8 3/4".
- Negative values: show the minus sign before the feet: -3' 7".

**Input parsing for compound units**:
The input parser must accept all of these as equivalent representations of the same length:
- "5'4"" or "5' 4""
- "5ft 4in"
- "5.333ft"
- "5.333'"
- "64in"
- "64""
- "1.6256m"
- "162.56cm"
- "1625.6mm"

The parser identifies the unit suffix and converts to meters. If no suffix is provided, the value is interpreted in the current display unit.

### 2.6 Angular Units

Angles are a separate dimension from length but share the same two-layer pattern:

| Unit | Abbreviation | Per Revolution | Internal |
|---|---|---|---|
| Degrees | deg / ° | 360 | Radians |
| Radians | rad | 2*pi | Radians |
| Gradians | grad | 400 | Radians |
| Turns | turn | 1 | Radians |
| Arc minutes | arcmin / ' | 21600 | Radians |
| Arc seconds | arcsec / " | 1296000 | Radians |

Internal angular storage is radians. Display is degrees by default, configurable per profile.

### 2.7 Area and Volume Units

When the editor displays area or volume (e.g., face area, room volume, terrain coverage), it uses the square or cube of the display unit:

- Display unit = mm --> area in mm^2, volume in mm^3
- Display unit = ft --> area in ft^2 (sq ft), volume in ft^3 (cu ft)
- Display unit = m --> area in m^2, volume in m^3

Special-case area units like acres, hectares, and square miles may be offered as alternatives in terrain/GIS profiles.

### 2.8 Rate and Velocity Units

For physics-aware contexts (animation, simulation, flight):

| Display Unit | Velocity | Acceleration |
|---|---|---|
| m | m/s | m/s^2 |
| km | km/h | km/h/s |
| ft | ft/s | ft/s^2 |
| nmi | knots (nmi/h) | knots/s |
| mi | mph | mph/s |

The internal representation is always m/s and m/s^2.

---

## Part 3: Unit Profiles / Presets

### 3.1 Purpose

A unit profile is a named bundle of settings that configures the entire unit/grid/snap system for a particular workflow. Selecting a profile instantly reconfigures display units, grid spacing, snap increments, decimal precision, and ruler behavior.

Profiles eliminate the need for users to manually configure dozens of settings. A mechanical engineer selects "CAD (mm)" and gets millimeter display, 0.1mm snap, 10mm major grid, 3-decimal precision -- all in one click.

### 3.2 Preset Profiles

| Profile Name | Display Unit | Grid Snap | Grid Major | Decimal Precision | Fractional Precision | Typical Use Case |
|---|---|---|---|---|---|---|
| **Game (Metric)** | m | 0.25 m | 1 m | 3 | -- | Unity, Godot, general game dev |
| **Game (Unreal)** | cm | 5 cm | 100 cm | 1 | -- | Unreal Engine content |
| **Game (Roblox)** | stud | 1 stud | 8 studs | 1 | -- | Roblox content |
| **CAD (mm)** | mm | 0.1 mm | 10 mm | 3 | -- | Mechanical engineering, 3D printing |
| **CAD (inch)** | in | 1/16 in | 1 in | 4 | 1/64 | US mechanical engineering |
| **Architecture (Metric)** | m | 0.1 m | 1 m | 2 | -- | Metric buildings, interiors |
| **Architecture (Imperial)** | ft + in | 1 in | 1 ft | -- | 1/16 | US buildings, interiors |
| **Minecraft / Voxel** | blocks | 1 block | 16 blocks | 0 | -- | Voxel games, block worlds |
| **Terrain / GIS** | m | 10 m | 100 m | 1 | -- | Large worlds, terrain sculpting |
| **Jewelry / Watchmaking** | mm | 0.01 mm | 1 mm | 3 | -- | Small precision objects |
| **Aviation** | ft (alt) / nmi (dist) | 100 ft | 1000 ft | 0 | -- | Flight simulators |
| **Golf / Sports** | yd | 1 yd | 10 yd | 1 | -- | Sports game levels |
| **Pixel Art** | px (16 px/m) | 1 px | 16 px | 0 | -- | Pixel-art 2D games |
| **2D Game** | px (100 px/m) | 1 px | 100 px | 0 | -- | Standard 2D games |
| **Urban Planning** | m | 1 m | 100 m | 0 | -- | City-scale layout |
| **Precision Machining** | thou | 1 thou | 10 thou | 1 | -- | CNC, milling |
| **Custom** | (user-defined) | (user) | (user) | (user) | (user) | Any specialized workflow |

### 3.3 Profile Structure

Each profile defines:

- **display_unit**: The primary unit shown everywhere.
- **secondary_unit**: An optional second unit shown in tooltips and the inspector for cross-reference (e.g., inches alongside mm).
- **grid_snap**: The base grid spacing in display units.
- **grid_major**: How many grid lines between major (thicker) lines.
- **decimal_precision**: Number of decimal places for display (metric units).
- **fractional_precision**: Denominator for fractional display (imperial units, power of 2).
- **angular_unit**: Degrees, radians, or gradians.
- **angular_snap**: Default angular snap increment in angular units.
- **pixels_per_meter**: Only relevant when display_unit is px.

### 3.4 Profile Lifecycle

- **Project-level**: Each project has a default profile set at creation time. This is the source of truth.
- **User override**: A user can temporarily override the project profile for their own viewport (e.g., checking dimensions in inches while the project uses mm). This override does not affect the saved file.
- **Viewport-local**: In a multi-viewport layout, each viewport could theoretically have its own display unit, though this is an advanced feature.
- **Profile switching**: Changing profiles does not modify any geometry. It only changes how numbers are displayed and how input is interpreted. All internal data remains in meters.

### 3.5 Custom Profiles

Users can create and save custom profiles. A custom profile is a full specification of all profile fields. Custom profiles are stored per-user (not per-project) so they carry across projects. A project can reference a custom profile by name.

### 3.6 Target Engine Export Profiles

When exporting content for a specific engine, the export pipeline uses a target profile to determine the output scale factor:

| Target | Export Scale | Notes |
|---|---|---|
| Unity | 1:1 (meters) | Direct match to internal |
| Unreal Engine | x100 (centimeters) | Multiply all positions by 100 |
| Godot 3D | 1:1 (meters) | Direct match |
| Godot 2D | x(pixels_per_meter) | Multiply by PPM |
| Roblox | x(1/0.28) (studs) | Divide by 0.28 |
| glTF/USD | 1:1 (meters) | Standard interchange is meters |
| FBX | configurable | FBX header specifies unit |
| STL (3D print) | x1000 (mm) | STL convention is mm |

---

## Part 4: Chunk System

### 4.1 Motivation

Large worlds cannot exist as a single monolithic data structure. The chunk system partitions the world into discrete, independently loadable regions. This enables:

- **Streaming**: Load only nearby chunks; unload distant ones. Essential for open worlds.
- **Serialization**: Save and load individual chunks without touching the rest of the world.
- **Collaboration**: In multi-user editing, lock individual chunks to prevent conflicts.
- **LOD (Level of Detail)**: Distant chunks can use simplified representations.
- **Culling**: Quickly reject entire chunks that are outside the camera frustum.
- **Undo granularity**: Chunk-scoped undo histories for large worlds.
- **Parallel processing**: Different chunks can be processed on different threads.

### 4.2 Chunk Hierarchy

The world is divided into a three-level hierarchy:

| Level | Name | Default Size | Contains | Purpose |
|---|---|---|---|---|
| 0 | Cell | 1m x 1m x 1m | Raw geometry/voxels | Finest granularity for editing |
| 1 | Chunk | 16m x 16m x 16m | 16^3 cells | Basic load/save/stream unit |
| 2 | Region | 256m x 256m x 256m | 16^3 chunks | Disk file grouping, broad culling |

**All sizes are in internal meters.** When the display unit is blocks (1 block = 1 meter), a chunk is 16 blocks -- matching Minecraft's convention. When the display unit is centimeters (Unreal), a chunk is 1600cm.

Chunk dimensions are configurable per-project. Not all projects need the same granularity:

| Project Type | Suggested Chunk Size | Rationale |
|---|---|---|
| FPS game (indoor) | 8m x 8m x 4m | Small rooms, frequent doors/portals |
| Open-world game | 32m x 32m x 32m | Large terrain, fewer but bigger chunks |
| Voxel / Minecraft-like | 16m x 16m x 16m | Classic Minecraft convention |
| Architectural | 16m x 16m x 4m | Building floors are short vertically |
| Terrain-only | 64m x 64m x 1m | Very wide, flat heightmap regions |
| Jewelry / small CAD | 0.1m x 0.1m x 0.1m | Tiny objects, high density |
| Planetary-scale | 1000m x 1000m x 1000m | Extreme distance, coarse chunks |

### 4.3 Chunk Addressing

Chunks are addressed by integer coordinates (cx, cy, cz) relative to the world origin. The world-space position of a chunk's minimum corner is:

```
world_x = cx * chunk_width
world_y = cy * chunk_height
world_z = cz * chunk_depth
```

This means chunk (0, 0, 0) spans from (0, 0, 0) to (chunk_width, chunk_height, chunk_depth). Chunk (-1, 0, 0) spans from (-chunk_width, 0, 0) to (0, 0, chunk_depth).

### 4.4 Streaming Policy

| Parameter | Description | Default | Configurable |
|---|---|---|---|
| Stream radius | Load chunks within this distance of the camera | 8 chunks | Yes |
| Keep-alive radius | Don't unload chunks within this radius (hysteresis) | 10 chunks | Yes |
| Priority | Closer chunks load first | Distance-based | Yes |
| Budget | Max chunks to load per frame | 2 | Yes |
| Preload direction | Bias loading toward camera movement direction | Enabled | Yes |

Hysteresis (stream radius < keep-alive radius) prevents thrashing when the camera sits near a chunk boundary.

### 4.5 LOD Strategy per Chunk

| LOD Level | Distance (chunks) | Representation | Vertex Budget |
|---|---|---|---|
| 0 (full) | 0 - 4 | Full geometry | No limit |
| 1 | 4 - 8 | Simplified mesh (50% reduction) | 50% of LOD 0 |
| 2 | 8 - 16 | Billboard / impostor | Minimal |
| 3 | 16+ | Bounding box / silhouette | Trivial |

LOD transitions should be smooth (crossfade or morph) to avoid popping.

### 4.6 Chunk Visualization in the Editor

The editor provides visual aids for understanding chunk layout:

- **Chunk grid overlay**: Faint colored lines showing chunk boundaries in the 3D viewport. Toggle with a keyboard shortcut.
- **Active chunk highlight**: The chunk containing the cursor or selection gets a subtle highlight color.
- **Minimap**: A top-down view showing loaded chunks as filled squares, unloaded chunks as outlines, and the camera frustum as a wedge.
- **Chunk statistics panel**: Shows per-chunk data for the selected chunk:
  - Entity count
  - Triangle count
  - Texture memory
  - Load state (loaded, loading, unloaded, error)
  - Lock state (unlocked, locked by user X)
  - Dirty flag (unsaved changes)

### 4.7 Cross-Chunk Entities

Entities that span chunk boundaries (e.g., a long bridge) need special handling:

- **Home chunk**: Every entity has a home chunk determined by its origin/pivot point.
- **Boundary references**: Chunks maintain a list of entities from neighboring chunks that protrude into their volume.
- **Atomic loading**: When loading a chunk, also load boundary-referenced entities from neighbors (even if those neighbor chunks are otherwise unloaded).
- **Splitting policy**: Very large entities (spanning 3+ chunks) should be flagged for the user to consider splitting.

---

## Part 5: Ruler / Measurement Tools

### 5.1 On-Screen Rulers

Rulers are the thin graduated bars docked to the viewport edges, like those in Photoshop, Illustrator, or InDesign.

**Placement**: Top edge (horizontal) and left edge (vertical) of each 3D viewport. In a multi-viewport layout, each viewport gets its own rulers calibrated to that viewport's zoom and projection.

**Tick marks**:
- Ticks are labeled in the current display unit.
- The tick density adapts to zoom level. When zoomed out, only major ticks show. When zoomed in, progressively finer subdivisions appear.
- Subdivision logic:
  - Metric: subdivide by 10 (1m -> 10cm -> 1cm -> 1mm -> 0.1mm ...)
  - Imperial: subdivide by 2 (1' -> 6" -> 3" -> 1.5" ...) or by standard fractions (1" -> 1/2" -> 1/4" -> 1/8" -> 1/16")
  - Game units: subdivide by the grid major interval

**Guidelines**:
- Click and drag from a ruler to create a reference guideline (horizontal or vertical infinite line).
- Guidelines are non-printing, non-exporting visual aids.
- Double-click a guideline to type an exact position.
- Right-click a guideline to delete it or lock it.
- Guidelines snap to the current grid.

**Interaction**:
- Toggle ruler visibility: Ctrl+R (configurable).
- Right-click ruler for options: change unit, change subdivision, toggle guidelines.
- Hover over ruler to see exact world position at cursor.

### 5.2 Measurement Tool (Tape Measure)

A dedicated tool for measuring distances, angles, areas, and volumes in the scene.

**Distance measurement**:
- Click point A, click point B -> displays distance between them.
- The measurement line is drawn in the viewport with end markers and a centered label.
- The label shows the distance in the primary display unit.
- A tooltip or secondary label shows the same distance in the secondary display unit.
- Hold Shift to constrain to axis-aligned measurement (horizontal or vertical only).
- Right-click the measurement to see it in all available units simultaneously.

**Multi-point (polyline) measurement**:
- Click multiple points to measure cumulative path length.
- Each segment shows its individual length.
- The total path length is shown at the end.
- Useful for measuring along a curved path (approximated as segments).

**Angle measurement**:
- Click three points (A, B, C) to measure the angle at B (angle ABC).
- Displayed as an arc with the angle value in the current angular unit.
- Optional: show the supplementary angle as well.

**Area measurement**:
- Click points to define a polygon, then close it (click the first point or press Enter).
- The enclosed area is displayed in square display units.
- Concave polygons are supported.
- The polygon is shown as a semi-transparent overlay.

**Volume measurement**:
- Select a box region (two opposite corners) to measure a rectangular volume.
- Alternatively, select a closed mesh to compute its volume.
- Displayed in cubic display units.

**Measurement persistence**:
- Measurements remain visible as annotations until explicitly dismissed (click X on the label, or press Delete while hovering).
- A "Clear all measurements" option in the toolbar or right-click menu.
- Measurements do NOT save with the project -- they are ephemeral session aids.
- Future consideration: optionally save measurements as a layer for documentation.

### 5.3 Dimension Annotations

Unlike ephemeral measurements, dimension annotations are persistent, saved objects that form part of the project data. They behave like CAD dimension lines.

**Linear dimensions**:
- Horizontal dimension: measures horizontal distance, line is horizontal.
- Vertical dimension: measures vertical distance, line is vertical.
- Aligned dimension: measures true distance, line is parallel to the measured span.
- Each shows two witness lines, a dimension line with arrowheads (or ticks), and a text label centered on the dimension line.

**Angular dimensions**:
- Shows an arc between two lines/edges with the angle value.
- Supports acute, obtuse, and reflex angles.

**Radius and diameter dimensions**:
- For circular edges or cylindrical faces.
- Radius: leader line from center to circumference, labeled "R 25.00 mm".
- Diameter: line through center, labeled with diameter symbol.

**Ordinate dimensions**:
- Measures distance from a user-defined datum point along one axis.
- Common in manufacturing drawings.
- A series of ordinate dimensions share a common baseline.

**Leader lines with callouts**:
- An arrow pointing to a feature, connected by a line (or spline) to a text box.
- The text box contains user-typed notes or auto-generated data (material, weight, etc.).

**Live updating**:
- All dimension annotations update their displayed values in real time when the referenced geometry moves.
- If the referenced geometry is deleted, the annotation is flagged as "broken" with a visual indicator.

### 5.4 Cross-Reference Display

The editor provides multiple locations where unit values are shown, each with an opportunity to show cross-reference conversions:

**Status bar** (bottom of viewport):
- Always shows cursor world position in primary display unit: "X: 1250.000 mm Y: 0.000 mm Z: -3400.000 mm"
- If a secondary unit is configured, shows it in parentheses: "X: 1250.000 mm (49.213 in)"

**Inspector panel** (sidebar):
- Object position, rotation, and scale in primary display unit.
- A small toggle or hover-tooltip to see values in secondary unit.
- Object bounding box dimensions shown with primary and secondary.

**Measurement tool results**:
- Primary unit prominently displayed.
- Full cross-reference shown on hover or in a detail popover:
  "2540.000 mm = 2.540 m = 100.000 in = 8' 4" = 254.000 cm"

**Unit calculator panel** (optional, togglable):
- A small floating panel where the user types a value with any unit suffix.
- Instantly shows the equivalent in all other units.
- Acts as a quick-reference converter.
- Example: type "5'6"" and see:
  - 1.6764 m
  - 167.64 cm
  - 1676.4 mm
  - 66 in
  - 5.5 ft
  - 1.8333 yd

---

## Part 6: Grid System

### 6.1 Grid Types

The editor supports multiple grid geometries:

**Cartesian grid** (default):
- Square grid aligned to two world axes.
- Default plane: XZ (horizontal ground plane).
- Lines extend in both axes at regular intervals.
- Most common for game levels, architecture, and general 3D.

**Polar grid**:
- Concentric circles at radial intervals + radial lines at angular intervals.
- Centered on a user-defined origin.
- Useful for rotational parts: gears, wheels, turntables, clock faces.
- Snap modes: snap to radial distance, snap to angle, snap to intersection.

**Isometric grid**:
- Lines at 60-degree angles instead of 90 degrees.
- Used for isometric 2D game art and certain crystallographic layouts.
- Snap to isometric grid intersections.

**Triangular grid**:
- Equilateral triangles tiling the plane.
- Useful for hexagonal game maps (hex grids are the dual of triangular grids).

**Custom angle grid**:
- User-defined angle between axes.
- For specialized oblique projections or non-standard layouts.

### 6.2 Grid Properties

| Property | Description | Default |
|---|---|---|
| Snap spacing | Distance between snap-able grid points, in display units | Profile-dependent |
| Major interval | Every Nth line is drawn thicker/brighter | 10 (metric) or varies |
| Grid plane | Which world plane the grid lies on | XZ |
| Grid offset | Vertical offset of the grid plane | 0 |
| Grid extent | Finite radius or infinite | Infinite (fades with distance) |
| Line color | Color of minor grid lines | From editor theme |
| Major line color | Color of major grid lines | From editor theme |
| Opacity | Grid transparency | 0.3 |
| Fade distance | Distance at which grid lines fade out | 100 display units |
| Subdivision | Show finer grid lines when zoomed in | Enabled |
| Subdivision levels | How many times to subdivide beyond snap spacing | 2 |
| Axis indicators | Show colored lines for world axes (X=red, Y=green, Z=blue) | Enabled |

### 6.3 Adaptive Grid Density

The grid dynamically adjusts its visible density based on camera zoom level:

- **Zoomed out**: Only major grid lines are visible. Subdivisions are hidden because they would be too dense to be useful.
- **Normal view**: Major and minor grid lines visible. One subdivision level shown.
- **Zoomed in**: Full subdivision hierarchy visible. The finest visible level is always at least a few pixels apart on screen.

The algorithm:
1. Compute the screen-space size of one grid cell at the current zoom.
2. If the cell is smaller than a threshold (e.g., 8 pixels), hide that level and show only the next coarser level.
3. If the cell is larger than another threshold (e.g., 64 pixels), show the next finer subdivision.
4. Repeat recursively for each subdivision level.

This ensures the grid always looks clean regardless of zoom, with no overlapping lines or empty voids.

### 6.4 Snap System

Snapping constrains object movement, rotation, and scaling to discrete values. Multiple snap modes can be active simultaneously; the system picks the nearest valid snap point.

**Grid snap**:
- Constrain translation to grid intersections.
- The grid is defined in display units, so snap points are "nice" numbers in the user's unit system.
- Toggle: button in toolbar + keyboard shortcut.

**Object snap (O-snap)**:
- Snap to geometric features of other objects:
  - Vertex: corners of meshes
  - Edge midpoint: center of edges
  - Face center: centroid of faces
  - Object origin/pivot
  - Bounding box corners and edge midpoints
  - Circle center, quadrant points, tangent points
  - Intersection of two edges
  - Perpendicular foot (nearest point on an edge)
  - Extension (along the extension of an edge beyond its endpoint)
- Each O-snap type can be individually enabled/disabled.
- When the cursor is near a valid snap point, a snap indicator icon appears (diamond for vertex, triangle for midpoint, circle for center, etc.).
- Tooltip shows the snap type and coordinates.

**Increment snap**:
- Move by fixed increments (e.g., 0.5 display units at a time) but NOT aligned to the grid.
- The increment is relative to the starting position.
- Useful when you want consistent spacing but don't care about absolute grid alignment.

**Angular snap**:
- Constrain rotation to fixed angles.
- Common values: 1, 5, 10, 15, 30, 45, 90 degrees.
- User-configurable.

**Scale snap**:
- Constrain scaling to fixed increments: 0.01, 0.05, 0.1, 0.25, 0.5, 1.0.
- Or snap to "nice" percentages: 25%, 50%, 75%, 100%, 150%, 200%.

**Plane snap / constraint**:
- Lock movement to a specific plane (XY, XZ, YZ, or a custom plane).
- Activated by holding a modifier key or clicking a constraint button.

**Surface snap**:
- Snap an object to the surface of another object.
- The snapped object is positioned so its origin (or a specified point) lies on the target surface.
- Normal alignment optional: rotate the snapped object to align with the surface normal.

**Snap priority and conflict resolution**:
When multiple snap modes are active and multiple valid snap points are nearby:
1. Object snap takes priority over grid snap (snapping to a vertex is more intentional than snapping to the grid).
2. Closer snap points take priority over farther ones.
3. A configurable "snap radius" (in screen pixels) defines how close the cursor must be to trigger a snap.
4. A priority list (user-configurable) breaks ties.

### 6.5 Construction Planes

Beyond the default ground-plane grid, users can create temporary or persistent construction planes:

- **From face**: Create a grid aligned to a selected face.
- **From 3 points**: Define a plane by clicking three non-collinear points.
- **From object**: Align to an object's local XY, XZ, or YZ plane.
- **Named planes**: Save construction planes by name for reuse.
- **Active plane**: Only one construction plane is active at a time for grid/snap purposes. Switching is done via a dropdown or keyboard shortcut.

---

## Part 7: Precision & Floating Point

### 7.1 The Floating-Point Problem

IEEE 754 float32 has approximately 7 significant decimal digits. Float64 has approximately 15. This means:

| Distance from Origin | Float32 Precision | Float64 Precision |
|---|---|---|
| 1 m | ~0.0001 mm | ~0.0000000001 mm |
| 10 m | ~0.001 mm | ~0.000000001 mm |
| 100 m | ~0.01 mm | ~0.00000001 mm |
| 1 km | ~0.06 mm | ~0.0000001 mm |
| 10 km | ~0.6 mm | ~0.000001 mm |
| 100 km | ~6 mm | ~0.00001 mm |
| 1000 km | ~60 mm | ~0.0001 mm |
| 10,000 km | ~600 mm | ~0.001 mm |

For a CAD tool that promises sub-millimeter precision, float32 fails beyond about 1 km from the origin. For a game with large open worlds (GTA-scale: ~10 km), float32 causes visible rendering jitter.

Float64 is safe to ~10,000 km for sub-mm precision, which covers virtually all practical use cases.

### 7.2 Precision Requirements by Domain

| Domain | Required Precision | Typical World Extent | Minimum Viable |
|---|---|---|---|
| Jewelry / watch | 0.01 mm | 0.1 m | f32 sufficient |
| Mechanical CAD | 0.001 mm | 10 m | f32 sufficient |
| Architecture | 1 mm | 500 m | f32 marginal, f64 preferred |
| Game level (indoor) | 1 mm | 200 m | f32 sufficient |
| Game level (open world) | 1 cm | 10 km | f32 insufficient, need mitigation |
| Terrain / GIS | 10 cm | 100 km | f64 required |
| Planetary | 1 m | 10,000 km | f64 required |

### 7.3 Strategy: Layered Precision

Forge uses a layered approach, combining multiple techniques:

**Layer A: World coordinates in f64**

All entity positions (translation component of transforms) are stored as f64 (double precision). This is the authoritative position. It is used for:
- Serialization (save/load)
- Spatial queries and indexing
- Chunk assignment
- Distance calculations
- Editor UI display

Cost: 24 bytes per position instead of 12. Acceptable -- position data is a tiny fraction of total memory compared to mesh and texture data.

**Layer B: Camera-relative f32 for rendering**

Before sending data to the GPU (via wgpu), all positions are converted to camera-relative f32:

```
render_position_f32 = (world_position_f64 - camera_position_f64) as f32
```

Because the camera is always at (0, 0, 0) in the rendering coordinate system, all visible geometry is close to the origin, and f32 has maximum precision. This technique is used by virtually all large-world game engines (Star Citizen, Flight Simulator, No Man's Sky, etc.).

The conversion happens on the CPU, per frame, for all visible entities. The GPU never sees world-space coordinates.

**Layer C: Chunk-local coordinates for physics**

Physics simulation operates in chunk-local space. Each chunk has its own local origin at its minimum corner. Entity positions within a chunk are expressed as f32 offsets from the chunk origin. Since chunks are at most ~64m across (even with large chunk sizes), the maximum offset is small, and f32 is more than sufficient.

Cross-chunk physics interactions (e.g., a projectile crossing a chunk boundary) require coordinate transformations between chunk-local spaces. This is a simple subtraction/addition of chunk origins (done in f64, result cast to f32).

**Layer D: Fixed-point for heightmaps**

Terrain heightmaps use integer storage with a scale factor:

- Store heights as u16 or i16 (0 - 65535 range).
- Define a vertical scale: e.g., 1 unit = 0.1 m, giving a range of 0 - 6553.5 m.
- This provides uniform precision across the entire terrain (no precision degradation with distance from origin).
- The scale factor is project-configurable.

**Layer E: Floating origin (optional, for extreme worlds)**

For planetary-scale or space-sim projects where even f64 might approach limits (or where third-party libraries require f32 world positions):

- Periodically re-center the world origin to the camera position.
- All entity positions are shifted by the negation of the camera offset.
- This is transparent to the user and to most engine systems.
- Trigger: when the camera moves more than N meters from the current origin (N = 1000 or 10000).
- Caveat: all systems must handle the re-centering event (physics, particles, audio sources, etc.). This adds complexity and should only be enabled when needed.

### 7.4 Precision Display

The number of decimal places shown to the user is controlled by the unit profile (see Part 3). This is a display concern only -- internal storage always uses full f64 precision.

Rounding for display:
- Use "round half to even" (banker's rounding) to avoid statistical bias.
- For fractional inches, round to the nearest configured fraction (1/16, 1/32, etc.).
- Never show more decimal places than the precision warrants. Showing "1.000000000 mm" when the underlying value is only accurate to 0.01 mm is misleading.

### 7.5 Precision Warnings

The editor should warn the user when precision limits are approached:

- If an entity is placed more than 10 km from the origin (f32 rendering artifacts likely): show a warning icon on the entity and in the status bar.
- If the project has floating origin disabled and the camera is far from origin: suggest enabling it.
- If chunk-local physics coordinates exceed safe f32 range (very large chunks + edge positions): warn at chunk configuration time.

---

## Part 8: Scale Reference Objects

### 8.1 Purpose

One of the hardest things in 3D editing is comprehending scale. Is the room you built actually room-sized, or is it the size of a shoebox? Scale reference objects are built-in, non-editable meshes that can be placed in the scene as visual scale indicators.

### 8.2 Reference Object Library

| Object | Approximate Size | Category |
|---|---|---|
| Human figure (standing) | 1.75 m tall | People |
| Human figure (sitting) | 1.2 m tall | People |
| Human hand | 0.19 m long | People |
| Sedan car | 4.5 m L x 1.8 m W x 1.5 m H | Vehicles |
| Bicycle | 1.8 m L x 0.6 m W x 1.1 m H | Vehicles |
| Standard door | 2.1 m H x 0.9 m W | Architecture |
| Standard window | 1.2 m H x 1.0 m W | Architecture |
| Staircase (one flight) | 2.8 m H x 3.5 m L x 1.0 m W | Architecture |
| Standard room (box) | 4 m x 5 m x 2.7 m | Architecture |
| Basketball | 0.24 m diameter | Objects |
| Golf ball | 0.04267 m diameter | Objects |
| US quarter coin | 0.02426 m diameter | Objects |
| Sheet of paper (A4) | 0.210 m x 0.297 m | Objects |
| Sheet of paper (US Letter) | 0.216 m x 0.279 m | Objects |
| Smartphone | 0.15 m x 0.07 m x 0.008 m | Objects |
| Football field (American) | 91.44 m x 48.76 m | Terrain |
| Soccer pitch | 105 m x 68 m | Terrain |
| Tennis court | 23.77 m x 10.97 m | Terrain |
| City block (typical) | 100 m x 100 m | Terrain |
| Minecraft Steve | 1.8 m tall, 0.6 m wide | Game |
| Minecraft chunk | 16 m x 16 m x 256 m | Game |

### 8.3 Visualization

- Reference objects render as semi-transparent silhouettes (ghost-like).
- They use a distinct color (e.g., teal or orange) with ~30% opacity.
- They do not participate in physics, selection, or export.
- They render on top of the grid but behind scene geometry (or toggle to render in front).
- A small label floats near each reference object showing its name and primary dimension.

### 8.4 Interaction

- Accessed via a toolbar button or menu: Insert > Scale Reference > [choose object].
- Placed at the cursor position or at the world origin.
- Can be moved but not scaled or rotated (the whole point is that they are true-to-scale).
- Toggle all references on/off with a single hotkey.
- Remove individually by selecting and pressing Delete.

### 8.5 Custom Reference Objects

Users can define custom reference objects by specifying:
- A name
- A mesh (from file or from a scene object)
- The real-world size (to verify the mesh is at the correct scale)

This is useful for domain-specific references (e.g., a specific machine part at known dimensions).

---

## Part 9: Unit-Aware UI Patterns

### 9.1 Input Fields

All numeric input fields in the editor that represent physical dimensions are unit-aware.

**Display behavior**:
- The field shows the value in the current display unit.
- A unit suffix is shown as a non-editable label inside or adjacent to the field: "[1250.000] mm"
- When the field has focus, the suffix remains visible as context.

**Input behavior**:
- **No suffix**: The typed value is interpreted as the current display unit.
  - Display unit = mm, user types "500" -> 500 mm = 0.5 m internal.
- **With suffix**: The typed value is interpreted in the specified unit, regardless of the current display unit.
  - Display unit = mm, user types "2in" -> 2 inches = 0.0508 m internal -> displayed as "50.800 mm".
- **Compound imperial**: "5'4"" or "5ft 4in" is parsed as feet-and-inches.
- **Math expressions**: Basic arithmetic is supported:
  - "2 * 3.5mm" = 7.0 mm
  - "5ft + 4in" = 5.333... ft = 1.6256 m
  - "100mm / 3" = 33.333 mm
  - Parentheses: "(5 + 3) * 2cm" = 16 cm
  - Pi constant: "pi * 10mm" = 31.416 mm
- **Relative input**: Prefix with + or - to add/subtract from the current value:
  - Current value: 100 mm. User types "+5" -> 105 mm.
  - Current value: 100 mm. User types "-10mm" -> 90 mm.

**Validation**:
- Non-numeric input (after unit parsing) shows an error state (red border).
- Out-of-range values (e.g., negative length for a dimension that must be positive) show a warning.
- The original value is preserved until the user commits (Enter) or cancels (Escape).

### 9.2 Scrubbing

Numeric fields support click-and-drag to scrub values:

- Click and drag left/right to decrease/increase the value.
- The scrub increment matches the grid snap spacing by default.
- Hold Shift to scrub in finer increments (1/10 of snap spacing).
- Hold Ctrl to scrub in coarser increments (10x snap spacing).
- Scrubbing respects snap settings if snap is enabled.

### 9.3 Property Inspector Integration

The property inspector (sidebar panel showing selected object properties) is fully unit-aware:

**Transform section**:
- Position: X, Y, Z in display units.
- Rotation: in angular display unit (degrees by default).
- Scale: dimensionless ratio (1.0 = original size). Optionally show actual size in display units as a tooltip.

**Dimensions section** (for mesh objects):
- Bounding box width, height, depth in display units.
- If a secondary unit is configured, show it in smaller text or tooltip.

**Custom properties**:
- User-defined numeric properties can be tagged with a unit dimension (length, angle, area, volume, mass, time, etc.).
- Tagged properties are displayed and parsed in the appropriate display unit.

### 9.4 Viewport Overlay

Permanent on-screen information in the viewport:

**Unit indicator** (bottom-right corner):
- Shows the current display unit prominently: "mm" or "ft+in" or "blocks".
- Clicking opens a dropdown to quickly switch display units.

**Scale bar** (bottom-left corner):
- A horizontal bar of a known real-world length, like a map legend.
- Adapts to zoom level: always shows a "round" length in display units.
- Example at different zooms:
  - Zoomed out: the bar represents 100 m.
  - Normal: the bar represents 1 m.
  - Zoomed in: the bar represents 10 mm.
- The bar's screen length stays roughly constant (100-200 pixels); the label changes.

**Grid spacing label** (near the scale bar):
- Shows the current grid snap spacing: "Grid: 0.1 mm"

**Camera altitude / distance** (top-right corner):
- Shows the camera's distance from the grid plane: "Alt: 25.400 m"
- Or the focal distance to the orbit target: "Dist: 3.200 m"
- Displayed in the current display unit.

### 9.5 Context Menus

Right-clicking a numeric value in the inspector or a measurement label offers:

- **Copy value**: copies "1250.000 mm" to clipboard.
- **Copy value in...**: submenu with all units: "Copy as inches", "Copy as meters", etc.
- **Convert display to...**: temporarily show this specific value in a different unit (does not change the global display unit).
- **Go to unit settings**: opens the unit profile configuration.

---

## Part 10: Configuration File Format

### 10.1 Project-Level Configuration

Each Forge project has a unit configuration file that is saved with the project and shared among all collaborators. This ensures everyone sees the same units.

```toml
# forge_project_units.toml
# This file is part of the Forge project and should be version-controlled.

[units]
# Internal unit is always meters. This is not configurable.
# internal = "meter"

# Primary display unit. All UI shows values in this unit.
display = "millimeter"

# Secondary display unit for cross-reference. Shown in tooltips/inspector.
# Set to "none" to disable.
secondary_display = "inch"

# Decimal precision for display (metric/decimal units).
decimal_precision = 3

# Fractional precision for imperial compound display.
# Must be a power of 2: 2, 4, 8, 16, 32, 64.
fractional_precision = 16

# Angular display unit: "degree", "radian", "gradian"
angular_unit = "degree"

# Angular decimal precision
angular_precision = 2

# Pixels per meter (only relevant when display unit is "pixel")
pixels_per_meter = 100.0

[grid]
# Grid geometry: "cartesian", "polar", "isometric", "triangular", "custom"
type = "cartesian"

# Grid snap spacing in display units
snap_spacing = 1.0

# Major grid line interval (every N minor lines)
major_interval = 10

# Grid plane: "xz", "xy", "yz"
plane = "xz"

# Vertical offset of the grid plane in meters
plane_offset = 0.0

# Whether grid subdivides when zoomed in
show_subdivisions = true

# Number of subdivision levels
subdivision_levels = 2

# Whether axis indicator lines are shown
show_axes = true

# Grid visual style
[grid.style]
minor_color = [0.3, 0.3, 0.3, 0.15]   # RGBA
major_color = [0.5, 0.5, 0.5, 0.3]     # RGBA
x_axis_color = [0.8, 0.2, 0.2, 0.5]    # Red
y_axis_color = [0.2, 0.8, 0.2, 0.5]    # Green
z_axis_color = [0.2, 0.2, 0.8, 0.5]    # Blue
fade_distance = 100.0                   # In display units

[snap]
# Master snap toggle
enabled = true

# Individual snap modes
grid_snap = true
object_snap = true
increment_snap = false
surface_snap = false
plane_snap = false

# Snap radius in screen pixels (how close cursor must be to trigger snap)
snap_radius = 10

# Angular snap increment in angular display units
angular_snap = 15.0

# Scale snap increment
scale_snap = 0.25

# Object snap sub-modes
[snap.object_snap_modes]
vertex = true
edge_midpoint = true
face_center = true
object_origin = true
bounding_box = true
circle_center = true
intersection = true
perpendicular = false
tangent = false
extension = false

[chunks]
# Whether the chunk system is active
enabled = true

# Chunk dimensions in meters
size = [16.0, 16.0, 16.0]

# Streaming radius in number of chunks
stream_radius = 8

# Keep-alive radius (must be >= stream_radius)
keep_alive_radius = 10

# Number of LOD levels (1 = no LOD, just full detail)
lod_levels = 4

# Show chunk boundary overlay in viewport
show_boundaries = false

# Maximum chunks to load per frame
load_budget_per_frame = 2

[ruler]
# Whether rulers are visible
visible = true

# Which viewport edges have rulers: "top", "left", "both", "none"
position = "both"

# Whether guidelines can be dragged from rulers
guidelines_enabled = true

[precision]
# World coordinate storage precision
world_coordinates = "f64"

# Physics coordinate precision (within chunk-local space)
chunk_local = "f32"

# Whether floating origin is enabled (re-center world periodically)
floating_origin = false

# Distance threshold for floating origin re-centering (meters)
floating_origin_threshold = 10000.0

# Heightmap storage format: "u16", "i16", "f32"
heightmap_format = "u16"

# Heightmap vertical scale (meters per heightmap unit)
heightmap_vertical_scale = 0.1

[export]
# Default export target for unit scaling
default_target = "unity"

# Whether to warn when exporting to a target with different unit conventions
warn_unit_mismatch = true
```

### 10.2 User-Level Preferences

Separate from the project config, each user has personal preferences that do not affect the project:

```toml
# ~/.forge/user_units_prefs.toml
# Personal preferences. Not shared with collaborators.

[display]
# Personal override for display unit (empty = use project setting)
display_unit_override = ""

# Always show secondary unit in inspector
show_secondary_unit = true

# Show unit suffix in input fields
show_unit_suffix = true

[measurement]
# Default measurement tool behavior
auto_clear_measurements = false
measurement_color = [1.0, 0.8, 0.0, 1.0]   # Yellow
measurement_font_size = 14

[scale_references]
# Whether reference objects are shown
show_references = false
reference_color = [0.0, 0.8, 0.8, 0.3]    # Teal, 30% opacity
default_reference = "human_figure"

[custom_profiles]
# User-defined unit profiles (list)
# Each profile follows the same schema as the project [units] section

[[custom_profiles.profiles]]
name = "My CNC Profile"
display = "thou"
secondary_display = "mm"
decimal_precision = 1
grid_snap = 0.5      # in thou
grid_major = 10
angular_unit = "degree"
angular_snap = 5.0
```

---

## Part 11: Cross-Reference Table

### 11.1 Built-In Quick Conversion Reference

The editor includes a built-in reference table accessible via Help > Unit Cross-Reference (or a keyboard shortcut). This table shows common real-world dimensions in multiple unit systems side by side.

| Object / Context | Meters | Centimeters | Millimeters | Inches | Feet + Inches | Minecraft Blocks | Unreal Units |
|---|---|---|---|---|---|---|---|
| Human height | 1.75 | 175 | 1750 | 68.90 | 5' 8 7/8" | 1.75 | 175 |
| Standard door (height) | 2.10 | 210 | 2100 | 82.68 | 6' 10 11/16" | 2.10 | 210 |
| Standard door (width) | 0.90 | 90 | 900 | 35.43 | 2' 11 7/16" | 0.90 | 90 |
| Ceiling height | 2.70 | 270 | 2700 | 106.30 | 8' 10 5/16" | 2.70 | 270 |
| Room (width) | 4.00 | 400 | 4000 | 157.48 | 13' 1 1/2" | 4.00 | 400 |
| Room (depth) | 5.00 | 500 | 5000 | 196.85 | 16' 4 7/8" | 5.00 | 500 |
| Sedan car (length) | 4.50 | 450 | 4500 | 177.17 | 14' 9 3/16" | 4.50 | 450 |
| Sedan car (width) | 1.80 | 180 | 1800 | 70.87 | 5' 10 7/8" | 1.80 | 180 |
| Basketball (diameter) | 0.24 | 24 | 240 | 9.45 | 9 7/16" | 0.24 | 24 |
| Golf ball (diameter) | 0.0427 | 4.27 | 42.67 | 1.68 | 1 11/16" | 0.043 | 4.27 |
| US quarter (diameter) | 0.0243 | 2.43 | 24.26 | 0.955 | 15/16" | 0.024 | 2.43 |
| A4 paper (short edge) | 0.210 | 21.0 | 210 | 8.27 | 8 1/4" | 0.21 | 21.0 |
| Smartphone (height) | 0.150 | 15.0 | 150 | 5.91 | 5 15/16" | 0.15 | 15.0 |
| Football field (length) | 91.44 | 9144 | 91440 | 3600 | 300' 0" | 91.44 | 9144 |
| Soccer pitch (length) | 105 | 10500 | 105000 | 4134 | 344' 6" | 105 | 10500 |
| Tennis court (length) | 23.77 | 2377 | 23770 | 935.8 | 78' 0" | 23.77 | 2377 |
| Minecraft chunk (edge) | 16.00 | 1600 | 16000 | 629.92 | 52' 5 15/16" | 16.00 | 1600 |
| City block (typical) | 100 | 10000 | 100000 | 3937 | 328' 1" | 100 | 10000 |
| 1 km reference | 1000 | 100000 | 1000000 | 39370 | 3280' 10" | 1000 | 100000 |

### 11.2 Engine Compatibility Quick Reference

For users exporting to specific engines, a quick reference showing how Forge's internal meters map to the target engine:

| Forge (1 m) | Unity | Unreal | Godot 3D | Godot 2D (100 ppm) | Roblox | glTF/USD |
|---|---|---|---|---|---|---|
| 1.0 m | 1.0 unit | 100 units | 1.0 unit | 100 pixels | ~3.57 studs | 1.0 m |
| 0.01 m (1 cm) | 0.01 | 1.0 | 0.01 | 1 pixel | ~0.036 studs | 0.01 m |
| 0.001 m (1 mm) | 0.001 | 0.1 | 0.001 | 0.1 pixel | ~0.004 studs | 0.001 m |

---

## Part 12: Implementation Priority

### 12.1 Staged Rollout

The unit system is foundational infrastructure. It must be built before or alongside many other editor features. The following priority stages reflect dependencies and user impact.

**Stage 1 -- Foundation (P0, must-have for first usable build)**

| Feature | Description | Depends On |
|---|---|---|
| Unit enum + conversion factors | Define all supported units and their meters-per-unit constants. | Nothing |
| Display unit config | Project setting to choose display unit. Stored in project file. | Unit enum |
| Unit formatting | Convert internal meters to display string ("1250.000 mm"). | Unit enum |
| Unit parsing | Parse user input with optional unit suffix back to meters. | Unit enum |
| Grid rendering (Cartesian) | Draw a Cartesian grid on the XZ plane with correct spacing in display units. | Unit formatting |
| Grid snap | Snap translations to grid intersections in display unit space. | Grid rendering |
| Status bar cursor position | Show cursor world position in display unit in the status bar. | Unit formatting |

**Stage 2 -- Core Editor Usability (P1, needed for productive editing)**

| Feature | Description | Depends On |
|---|---|---|
| Unit-aware input fields | All inspector numeric fields accept unit suffixes and show current unit. | Unit parsing, unit formatting |
| Unit profiles / presets | Predefined profiles (Game Metric, CAD mm, Architecture Imperial, etc.). | Display unit config |
| Measurement tool (two-point) | Click two points, see distance in display unit. | Unit formatting |
| Scale bar | Viewport overlay showing reference length. | Unit formatting, viewport rendering |
| Increment snap | Move by fixed display-unit increments. | Snap system |
| Angular snap | Rotate by fixed degree increments. | Snap system |

**Stage 3 -- Professional Features (P2, expected by CAD / architecture users)**

| Feature | Description | Depends On |
|---|---|---|
| Chunk system | World partitioning, streaming, chunk-local coords. | f64 world coords |
| Rulers with guidelines | Photoshop-style rulers on viewport edges, draggable guidelines. | Unit formatting, viewport rendering |
| Cross-reference display | Show values in secondary unit in tooltips and inspector. | Unit formatting, unit profiles |
| Compound unit display (ft+in) | Properly format and parse feet-and-inches with fractions. | Unit formatting, unit parsing |
| Object snap (O-snap) | Snap to vertices, midpoints, centers, intersections. | Snap system, spatial queries |
| Multi-point measurement | Polyline distance, angle, area measurement. | Measurement tool |
| Grid plane switching | Switch grid to XY, YZ, or arbitrary plane. | Grid rendering |

**Stage 4 -- Advanced (P3, differentiating features)**

| Feature | Description | Depends On |
|---|---|---|
| Dimension annotations | Persistent CAD-style dimension lines with live updating. | Measurement tool, annotation system |
| Polar / isometric grids | Alternative grid geometries. | Grid rendering |
| f64 world coordinates | Double-precision positions for large worlds. | Core transform system |
| Camera-relative rendering | Subtract camera pos (f64) before sending f32 to GPU. | f64 world coords, wgpu pipeline |
| Scale reference objects | Built-in reference meshes (human, car, door). | Asset system, viewport rendering |
| Surface snap | Snap objects to the surface of other geometry. | Snap system, raycasting |
| Construction planes | User-defined grid planes from faces or points. | Grid system, selection |
| Math expression input | Parse "2*3.5mm + 1in" in input fields. | Unit parsing |

**Stage 5 -- Polish & Niche (P4, nice-to-have)**

| Feature | Description | Depends On |
|---|---|---|
| Unit calculator panel | Type any value, see all conversions. | Unit formatting |
| Custom reference objects | User-defined scale reference meshes. | Scale reference objects |
| Per-viewport display unit | Different unit in each viewport of a multi-viewport layout. | Unit profiles, viewport system |
| LOD for chunks | Distance-based detail reduction per chunk. | Chunk system |
| Floating origin | Re-center world origin for extreme-distance worlds. | f64 world coords, all systems |
| Volume measurement | Measure volume of selected region or mesh. | Measurement tool, mesh analysis |
| DMS angular display | Degrees-minutes-seconds for GIS use. | Angular unit system |
| Export unit scaling | Auto-scale geometry for target engine on export. | Export pipeline, unit profiles |

### 12.2 Dependency Graph (Simplified)

```
Unit Enum + Conversion Factors
  |
  +-> Unit Formatting
  |     |
  |     +-> Status Bar Display
  |     +-> Scale Bar
  |     +-> Rulers
  |     +-> Cross-Reference Display
  |     +-> Unit Calculator
  |
  +-> Unit Parsing
  |     |
  |     +-> Unit-Aware Input Fields
  |     +-> Math Expression Input
  |     +-> Compound Unit Parsing (ft+in)
  |
  +-> Grid Rendering (Cartesian)
  |     |
  |     +-> Grid Snap
  |     +-> Polar / Isometric Grid
  |     +-> Grid Plane Switching
  |     +-> Construction Planes
  |
  +-> Snap System
  |     |
  |     +-> Increment Snap
  |     +-> Angular Snap
  |     +-> Object Snap
  |     +-> Surface Snap
  |
  +-> Unit Profiles / Presets
  |     |
  |     +-> Export Unit Scaling
  |
  +-> Measurement Tool
  |     |
  |     +-> Multi-Point Measurement
  |     +-> Dimension Annotations
  |
  +-> f64 World Coordinates
        |
        +-> Camera-Relative Rendering
        +-> Chunk System
        +-> Floating Origin
```

---

## Part 13: Open Questions & Future Work

### 13.1 Open Design Questions

**Q1: Should the internal unit be truly fixed at meters, or should we allow project-level internal unit selection (like 3ds Max)?**

Arguments for fixed meters:
- Simplicity. One less thing to configure, one less source of bugs.
- Consistency across projects. Opening someone else's project never surprises you.
- SI is the international standard. Meters are the base unit.

Arguments for configurable internal:
- Some domains (semiconductor: nm, astronomy: AU) are extremely far from meters. Conversion factors introduce floating-point noise.
- 3ds Max supports this and professionals use it.

Current recommendation: Fixed meters. The conversion noise argument is largely addressed by f64 storage. Revisit only if real-world users report problems.

**Q2: How should we handle unit conflicts when importing assets?**

If a user imports an FBX file authored in centimeters into a project configured for millimeters:
- Option A: Auto-scale on import (detect the source unit from file metadata, convert).
- Option B: Ask the user at import time ("This file uses centimeters. Scale to match project units?").
- Option C: Import raw and let the user fix it.

Current recommendation: Option B. Auto-scaling is convenient but can silently introduce errors. Asking once is a small cost for safety.

**Q3: Should chunk sizes be uniform or variable?**

Uniform: All chunks are the same size. Simple addressing, simple streaming.
Variable: Octree-style subdivision. Chunks in detailed areas are smaller; chunks in empty areas are larger. More efficient for sparse worlds but significantly more complex.

Current recommendation: Uniform for Stage 3. Consider variable/octree for a later stage if streaming performance requires it.

**Q4: What happens when the user switches display units mid-project?**

Nothing changes in the data. All internal values remain in meters. Only the display layer updates. Grid snap spacing, however, might need to be adjusted -- switching from mm to inches with a snap of 1.0 would change from 1mm snap to 1in snap, which is a 25.4x change. Should the snap spacing auto-convert to the nearest "nice" value in the new unit? Or should it carry over literally?

Current recommendation: Auto-convert snap to the nearest standard snap value for the new unit. Present the conversion to the user with an option to keep the literal value.

**Q5: How do we handle mixed-unit collaboration?**

User A prefers millimeters. User B prefers inches. They work on the same project.

The project file specifies a canonical display unit (the project default). Each user can override the display unit locally without affecting the saved project. All saved values are in meters, so there is no conflict.

Grid snap is trickier: if User A sets a 1mm grid and User B sets a 1/32" grid, their snapped positions will not align. This is an acceptable tradeoff -- the project owner can enforce a standard snap via the project config, and users should be aware that overriding it may cause alignment issues.

### 13.2 Future Considerations

**Parametric / constraint-driven dimensions**: In a fully parametric CAD system, dimensions are not just displayed values but are inputs to a constraint solver. The unit system must integrate with the constraint engine so that a dimension of "50 mm" constrains the geometry to exactly 0.05 m. This is far beyond the scope of the initial implementation but should not be precluded by the architecture.

**Non-linear coordinate systems**: GIS applications use latitude/longitude (angular coordinates on a sphere) and various map projections (UTM, Mercator, etc.). Supporting these would require a coordinate system layer beyond simple linear units. This is a specialized extension, not part of the core unit system.

**Time units**: Animation and simulation involve time. The same two-layer pattern applies: internal time in seconds, display in seconds, frames (at a given FPS), timecode (HH:MM:SS:FF), or milliseconds. This is a separate but analogous system.

**Mass, force, and other physical quantities**: If Forge adds integrated physics authoring (not just runtime simulation), it may need mass (kg), force (N), pressure (Pa), temperature (K/C/F), etc. Each would follow the same pattern: one internal SI unit, configurable display unit.

**Localization of unit display**: Some locales use comma as decimal separator (1.234,56 in Germany vs 1,234.56 in US). The formatting layer must be locale-aware.

**Accessibility**: Screen readers should announce unit values clearly: "one thousand two hundred fifty millimeters" not "one two five zero point zero zero zero em em". The unit-aware input fields should have proper ARIA labels indicating the unit context.

---

## Appendix A: Conversion Factor Reference

For implementors, the complete set of conversion factors (meters per unit):

| Unit ID | Meters per Unit | Exact? |
|---|---|---|
| nanometer | 1e-9 | Yes |
| micrometer | 1e-6 | Yes |
| millimeter | 1e-3 | Yes |
| centimeter | 1e-2 | Yes |
| decimeter | 1e-1 | Yes |
| meter | 1.0 | Yes |
| kilometer | 1e3 | Yes |
| thou | 2.54e-5 | Yes (by definition) |
| inch | 2.54e-2 | Yes (by definition) |
| foot | 3.048e-1 | Yes (12 inches) |
| yard | 9.144e-1 | Yes (3 feet) |
| mile | 1.609344e3 | Yes (5280 feet) |
| nautical_mile | 1.852e3 | Yes (by definition) |
| fathom | 1.8288 | Yes (6 feet) |
| minecraft_block | 1.0 | By convention |
| unreal_unit | 1e-2 | By convention |
| roblox_stud | 0.28 | Approximate (Roblox is ambiguous) |
| pixel | 1.0 / pixels_per_meter | Configurable |

All "Yes (by definition)" values are internationally standardized exact conversions. The inch is defined as exactly 25.4 mm (since 1959). All other imperial length units derive from the inch.

---

## Appendix B: Glossary

| Term | Definition |
|---|---|
| Internal unit | The unit used for all engine-internal math: meters. Never visible to user. |
| Display unit | The unit shown to the user in all UI elements. User-configurable. |
| Unit profile | A named preset that configures display unit, grid, snap, and precision together. |
| Compound unit | A display format combining two units (e.g., feet + inches). |
| Grid snap | Constraining movement to grid intersection points. |
| O-snap | Object snap: constraining to geometric features of existing objects. |
| Chunk | A discrete, independently loadable region of the world. |
| Region | A group of chunks, used for file organization and broad culling. |
| Floating origin | Technique of periodically re-centering the world origin to the camera to maintain f32 precision. |
| Camera-relative rendering | Converting world positions to camera-relative before sending to GPU, maintaining f32 precision near the camera. |
| Construction plane | A user-defined plane that serves as a temporary grid/snap reference. |
| Datum | A reference point or plane from which measurements are taken (used in ordinate dimensions). |
| PPM | Pixels per meter: configurable ratio for the pixel display unit. |
| Witness line | The thin lines extending from geometry to a dimension line (also called extension lines). |
| Coordinate system | Convention defining which axis is up, which is forward, and handedness (left/right). |
| Y-Up | Coordinate convention where the Y axis points up. Used by OpenGL, glTF, Blender, Unity, Godot. |
| Z-Up | Coordinate convention where the Z axis points up. Used by 3ds Max, Unreal, AutoCAD, Rhino. |
| Handedness | Whether a coordinate system is left-handed or right-handed, determined by cross-product direction. |
| Angular unit | Unit of measurement for angles: degrees, radians, gradians, turns, etc. |
| DMS | Degrees-minutes-seconds notation for angles (e.g., 45°30'15"). |
| SMPTE timecode | Standard time display format for film/broadcast: HH:MM:SS:FF (hours, minutes, seconds, frames). |
| Frame rate | Number of animation frames per second (e.g., 24fps for film, 60fps for games). |
| Sub-frame interpolation | Computing in-between values within a single frame for smoother playback at higher rates. |
| Linear RGB | Color space where values are proportional to physical light intensity, used for rendering math. |
| sRGB | Standard color space for displays, with gamma curve applied for perceptual uniformity. |
| HDR color | Color values exceeding the 0.0-1.0 range, used for emissive materials and light sources. |
| OKLCH | Perceptually uniform color space using Lightness, Chroma, and Hue components. |
| PPU | Pixels per unit: ratio mapping world units to pixel dimensions in 2D mode. |
| CPlane | Construction plane: a user-defined reference plane for sketching, snapping, and grid display. |
| Smart unit detection | Automatic inference of source file units based on coordinate ranges, file type, and metadata. |
| Measurement bookmark | A saved measurement with custom label, kept for reference or export. |
| Export unit mapping | Scale conversion applied when exporting to a target engine or file format. |

---

## Part 14: Coordinate System Modes

Different industries use different coordinate conventions:

| System | Up Axis | Forward | Right | Used By |
|--------|---------|---------|-------|---------|
| Y-Up Right-Hand | +Y | -Z | +X | OpenGL, Blender, Godot, glTF |
| Y-Up Left-Hand | +Y | +Z | +X | Unity, DirectX |
| Z-Up Right-Hand | +Z | +Y | +X | 3ds Max, Unreal, AutoCAD, Rhino |
| Z-Up Left-Hand | +Z | +X | +Y | Some CAD tools |

GameCAD should:
- Default to Y-Up Right-Hand (matching glTF/wgpu)
- Allow project-level override for Z-Up workflows
- Auto-convert on import from DCC tools (Blender Y-Up, Max Z-Up, etc.)
- Show axis indicator in viewport matching current convention
- Config option in project settings

---

## Part 15: Angular Units

| Unit | Symbol | Per Revolution | Use Case |
|------|--------|---------------|----------|
| Degrees | ° | 360 | General, games, architecture |
| Radians | rad | 2π | Math, physics, shaders |
| Gradians | grad | 400 | European surveying |
| Turns | turn | 1 | Procedural, animation cycles |
| Minutes of arc | ' | 21600 | Astronomy, surveying |
| Seconds of arc | " | 1296000 | Precision astronomy |
| Mils (NATO) | mil | 6400 | Military targeting |

- Internal: radians (Rust/glam standard)
- Display: degrees by default, configurable
- Inspector rotation fields always show in display angular unit
- Support DMS (degrees-minutes-seconds) input: "45°30'15""

---

## Part 16: Time Units for Animation

| Unit | Use Case | Frames @24 | Frames @30 | Frames @60 |
|------|----------|-----------|-----------|-----------|
| Seconds | Universal timeline | 24 | 30 | 60 |
| Frames | Animation, VFX | 1 | 1 | 1 |
| SMPTE Timecode | Film/broadcast | HH:MM:SS:FF | HH:MM:SS:FF | HH:MM:SS:FF |
| Beats/BPM | Music/rhythm games | Variable | Variable | Variable |
| Ticks | Simulation | Engine-defined | Engine-defined | Engine-defined |

- Timeline ruler shows in selected time unit
- Configurable frame rate per project (24, 25, 30, 48, 60, 120, custom)
- Timecode display mode for film/broadcast workflows
- Sub-frame interpolation for 60fps playback of 24fps animation

---

## Part 17: Velocity & Physics Units

| Quantity | Internal | Display Options |
|----------|----------|----------------|
| Linear velocity | m/s | m/s, km/h, mph, knots, ft/s |
| Angular velocity | rad/s | °/s, RPM, rad/s |
| Acceleration | m/s² | m/s², g-force, ft/s² |
| Force | N | N, kN, lbf, kgf |
| Mass | kg | kg, g, lb, oz, ton |
| Torque | N·m | N·m, lb·ft, kgf·m |
| Pressure | Pa | Pa, kPa, psi, bar, atm |
| Temperature | K | K, °C, °F |
| Density | kg/m³ | kg/m³, g/cm³, lb/ft³ |

For physics simulation preview — show velocities and forces in user-chosen units.

---

## Part 18: Color Units & Spaces

| Space | Components | Range | Use Case |
|-------|-----------|-------|----------|
| sRGB | R, G, B | 0-255 or 0.0-1.0 | Display, textures, UI |
| Linear RGB | R, G, B | 0.0-1.0+ (HDR) | Rendering, lighting math |
| HSV/HSB | H°, S%, V% | H:0-360, S:0-100, V:0-100 | Color picking (intuitive) |
| HSL | H°, S%, L% | H:0-360, S:0-100, L:0-100 | CSS, web design |
| OKLCH | L, C, H | Perceptual | Modern perceptual color |
| CIE XYZ | X, Y, Z | 0.0-1.0+ | Color science reference |
| Hex | #RRGGBB | 00-FF per channel | Web, CSS, config files |

- Color picker should show multiple formats simultaneously
- Copy color as hex, RGB tuple, or float array
- Warn when picking non-sRGB colors for UI elements
- Support HDR color values (>1.0) for emissive/light colors

---

## Part 19: Area & Volume Display

| Dimension | Display Options |
|-----------|----------------|
| Area | m², cm², mm², ft², in², yd², acres, hectares, km² |
| Volume | m³, cm³, mm³, L (liters), mL, gal, ft³, in³ |

- Auto-select appropriate unit based on magnitude:
  - < 0.01 m² → cm² or mm²
  - 0.01 - 10000 m² → m²
  - > 10000 m² → hectares or km²
- Show area/volume in measurement tool results
- Terrain area calculation for selected region
- Room volume estimation for acoustic simulation preview

---

## Part 20: DPI / Pixels Per Unit (2D Mode)

For 2D game development and UI design:

| Setting | Description | Default |
|---------|-------------|---------|
| PPU (Pixels Per Unit) | How many pixels fit in one world unit | 100 |
| Reference resolution | Target screen resolution | 1920x1080 |
| Pixel grid | Snap to pixel boundaries | ON in 2D mode |
| Sprite scale | Base scale factor for imported sprites | 1.0 |

- 2D mode available in any workspace tab
- Ortho camera with pixel-perfect grid
- Import sprites with auto-PPU detection from image dimensions
- Pixel snapping for crisp rendering at integer positions

---

## Part 21: Export Unit Mapping

When exporting to different engines/formats:

| Target | Expected Unit | Conversion from meters |
|--------|--------------|----------------------|
| Unreal Engine (.fbx) | 1 cm | × 100 |
| Unity (.fbx/.glb) | 1 m | × 1 (none) |
| Godot (.glb) | 1 m | × 1 (none) |
| 3ds Max (.fbx) | System units (configurable) | Based on target setting |
| AutoCAD (.dxf) | mm | × 1000 |
| 3D Printing (.stl) | mm | × 1000 |
| Web/Three.js (.glb) | 1 m | × 1 (none) |
| Minecraft (schematic) | 1 block = 1 m | × 1, quantize to integers |

- Export dialog shows unit mapping preview
- Auto-apply scale factor on export
- "Export for Unreal" preset: scale ×100, Z-up, FBX format
- Warn if exported geometry is unreasonably small/large for target

---

## Part 22: Measurement History & Bookmarks

- All measurements saved in a measurement log panel
- Each entry: timestamp, type (distance/angle/area), value, endpoints
- Bookmark important measurements with custom labels
- Export measurement report as CSV or PDF
- Compare measurements between versions (before/after editing)
- "Pin" a measurement to keep it visible as an overlay

---

## Part 23: Smart Unit Detection

Auto-detect likely unit system from imported files:

| Clue | Detected Unit | Confidence |
|------|--------------|------------|
| All coordinates < 10 | Meters (game asset) | Medium |
| All coordinates 100-10000 | Centimeters (Unreal) | High |
| All coordinates > 100000 | Millimeters (CAD) | High |
| File has ".dwg" extension | mm (AutoCAD) | High |
| File has ".blend" metadata | Blender unit setting | High |
| glTF with generator "Blender" | Meters | High |
| FBX with Max header | Check system unit metadata | High |

- Show "Detected unit: mm — does this look correct?" on import
- Preview imported model with scale reference (human figure)
- Allow manual override if detection is wrong
- Remember per-source corrections for future imports

---

## Part 24: Construction Plane System (Rhino-inspired)

| Feature | Description |
|---------|-------------|
| CPlane | Custom construction plane for sketching and snapping |
| World CPlane | Default XZ ground plane |
| Set CPlane | Click 3 points to define a custom plane |
| Named CPlanes | Save/recall custom planes by name |
| CPlane to Object | Align construction plane to selected face |
| CPlane to View | Set CPlane perpendicular to current view |

- Grid draws on active CPlane (not just world XZ)
- All grid snapping relative to CPlane
- Sketch tools (future) operate on CPlane
- Visual indicator showing CPlane orientation
- CPlane stored per-viewport in multi-viewport setups

---

*End of design document.*
