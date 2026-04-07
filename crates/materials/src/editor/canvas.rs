//! Main egui rendering: zoom/pan, nodes, wires.

use egui::{
    pos2, vec2, Color32, CornerRadius, CursorIcon, Id, Pos2, Rect, Sense, Stroke, StrokeKind, Ui,
};

use crate::graph::{
    MaterialGraph, MaterialNode, NodeId, NodeKind, PinDirection, PinId, PinType,
};

use super::state::{DragWire, NodeEditorState};

// ─────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────

const NODE_WIDTH: f32 = 180.0;
const TITLE_HEIGHT: f32 = 28.0;
const PIN_ROW_HEIGHT: f32 = 22.0;
const PIN_RADIUS: f32 = 6.0;
const ROUNDING: f32 = 6.0;

const ACCENT_COLOR: Color32 = Color32::from_rgb(0x4e, 0xff, 0x93);
const NODE_BG: Color32 = Color32::from_rgb(0x2a, 0x2a, 0x2a);
const CANVAS_BG: Color32 = Color32::from_rgb(0x1a, 0x1a, 0x1a);
const GRID_COLOR: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
const WIRE_COLOR: Color32 = Color32::from_rgb(0xcc, 0xcc, 0xcc);

// ─────────────────────────────────────────────────────────────────────
// Color helpers
// ─────────────────────────────────────────────────────────────────────

/// Map a node kind to the Crystalline-themed title bar color.
fn title_bar_color(kind: NodeKind) -> Color32 {
    match kind {
        NodeKind::PbrOutput => Color32::from_rgb(0x4e, 0xff, 0x93),
        NodeKind::MathAdd | NodeKind::MathMultiply | NodeKind::MathMix | NodeKind::MathLerp => {
            Color32::from_rgb(0x3e, 0x55, 0xff)
        }
        NodeKind::TextureSample | NodeKind::NormalMap => Color32::from_rgb(0x9b, 0x59, 0xb6),
        NodeKind::ConstantColor | NodeKind::ConstantFloat | NodeKind::ConstantVec3 => {
            Color32::from_rgb(0xe6, 0x7e, 0x22)
        }
        NodeKind::Fresnel | NodeKind::NoisePerlin | NodeKind::NoiseVoronoi => {
            Color32::from_rgb(0x3e, 0x55, 0xff)
        }
    }
}

/// Map a pin data type to its display color.
fn pin_color(pin_type: PinType) -> Color32 {
    match pin_type {
        PinType::Float => Color32::from_rgb(0xa0, 0xa0, 0xa0),
        PinType::Vec2 => Color32::from_rgb(0x00, 0xcc, 0xcc),
        PinType::Vec3 => Color32::from_rgb(0xff, 0xff, 0x00),
        PinType::Vec4 => Color32::from_rgb(0xff, 0xaa, 0x00),
        PinType::Color => Color32::from_rgb(0xff, 0xff, 0xff),
        PinType::Texture => Color32::from_rgb(0x9b, 0x59, 0xb6),
        PinType::Shader => Color32::from_rgb(0x00, 0xff, 0x00),
    }
}

// ─────────────────────────────────────────────────────────────────────
// Coordinate helpers
// ─────────────────────────────────────────────────────────────────────

/// Convert graph-space coordinates to screen-space pixels.
fn graph_to_screen(pos: glam::Vec2, offset: glam::Vec2, zoom: f32, canvas_min: Pos2) -> Pos2 {
    pos2(
        (pos.x + offset.x) * zoom + canvas_min.x,
        (pos.y + offset.y) * zoom + canvas_min.y,
    )
}

/// Convert screen-space pixels back to graph-space coordinates.
fn screen_to_graph(screen: Pos2, offset: glam::Vec2, zoom: f32, canvas_min: Pos2) -> glam::Vec2 {
    glam::Vec2::new(
        (screen.x - canvas_min.x) / zoom - offset.x,
        (screen.y - canvas_min.y) / zoom - offset.y,
    )
}

/// Compute the visual height of a node based on its pin count.
fn node_height(node: &MaterialNode) -> f32 {
    TITLE_HEIGHT + node.pins.len() as f32 * PIN_ROW_HEIGHT + 4.0
}

/// Compute the screen-space position of a pin circle.
fn pin_screen_pos(
    node: &MaterialNode,
    pin_index: usize,
    direction: PinDirection,
    offset: glam::Vec2,
    zoom: f32,
    canvas_min: Pos2,
) -> Pos2 {
    let base = graph_to_screen(node.position, offset, zoom, canvas_min);
    let x = match direction {
        PinDirection::Input => base.x,
        PinDirection::Output => base.x + NODE_WIDTH * zoom,
    };
    let y = base.y + (TITLE_HEIGHT + pin_index as f32 * PIN_ROW_HEIGHT + PIN_ROW_HEIGHT * 0.5) * zoom;
    pos2(x, y)
}

// ─────────────────────────────────────────────────────────────────────
// Main render function
// ─────────────────────────────────────────────────────────────────────

/// Render the full node graph editor into the given `Ui`.
pub fn show(ui: &mut Ui, state: &mut NodeEditorState) {
    let (response, painter) =
        ui.allocate_painter(ui.available_size_before_wrap(), Sense::click_and_drag());
    let canvas_rect = response.rect;
    let canvas_min = canvas_rect.min;

    // Background
    painter.rect_filled(canvas_rect, CornerRadius::ZERO, CANVAS_BG);

    // Grid
    let grid_spacing = 40.0 * state.zoom;
    if grid_spacing > 4.0 {
        let ox = (state.camera_offset.x * state.zoom) % grid_spacing;
        let oy = (state.camera_offset.y * state.zoom) % grid_spacing;
        let mut x = canvas_min.x + ox;
        while x < canvas_rect.max.x {
            painter.line_segment(
                [pos2(x, canvas_min.y), pos2(x, canvas_rect.max.y)],
                Stroke::new(1.0, GRID_COLOR),
            );
            x += grid_spacing;
        }
        let mut y = canvas_min.y + oy;
        while y < canvas_rect.max.y {
            painter.line_segment(
                [pos2(canvas_min.x, y), pos2(canvas_rect.max.x, y)],
                Stroke::new(1.0, GRID_COLOR),
            );
            y += grid_spacing;
        }
    }

    // Handle pan (middle-click drag)
    if response.dragged_by(egui::PointerButton::Middle) {
        let delta = response.drag_delta();
        state.camera_offset.x += delta.x / state.zoom;
        state.camera_offset.y += delta.y / state.zoom;
    }

    // Handle zoom (scroll)
    let scroll = ui.input(|i| i.raw_scroll_delta.y);
    if scroll != 0.0 && response.hovered() {
        let factor = if scroll > 0.0 { 1.1 } else { 1.0 / 1.1 };
        state.zoom = (state.zoom * factor).clamp(0.2, 3.0);
    }

    // Collect node ids to iterate
    let node_ids: Vec<NodeId> = state.graph.nodes.keys().copied().collect();

    // ── Draw connections ────────────────────────────────────────────
    // Build a snapshot of the pin positions we need
    let connections: Vec<_> = state.graph.connections.clone();
    for conn in &connections {
        let from_pos = pin_pos_for_connection(
            &state.graph,
            conn.from_node,
            conn.from_pin,
            state.camera_offset,
            state.zoom,
            canvas_min,
        );
        let to_pos = pin_pos_for_connection(
            &state.graph,
            conn.to_node,
            conn.to_pin,
            state.camera_offset,
            state.zoom,
            canvas_min,
        );

        if let (Some(fp), Some(tp)) = (from_pos, to_pos) {
            draw_bezier_wire(&painter, fp, tp, WIRE_COLOR);
        }
    }

    // ── Draw dragging wire ──────────────────────────────────────────
    if let Some(ref dw) = state.dragging_wire {
        let from_pos = pin_pos_for_connection(
            &state.graph,
            dw.from_node,
            dw.from_pin,
            state.camera_offset,
            state.zoom,
            canvas_min,
        );
        if let Some(fp) = from_pos {
            let tp = graph_to_screen(dw.current_pos, state.camera_offset, state.zoom, canvas_min);
            draw_bezier_wire(&painter, fp, tp, Color32::from_rgb(0x4e, 0xff, 0x93));
        }
    }

    // ── Draw and interact with nodes ────────────────────────────────
    let mut clicked_node: Option<NodeId> = None;
    let mut drag_started: Option<(NodeId, PinId)> = None;
    let mut drag_ended_pin: Option<(NodeId, PinId)> = None;

    for &nid in &node_ids {
        let node = match state.graph.nodes.get(&nid) {
            Some(n) => n,
            None => continue,
        };

        let screen_pos = graph_to_screen(node.position, state.camera_offset, state.zoom, canvas_min);
        let h = node_height(node) * state.zoom;
        let w = NODE_WIDTH * state.zoom;
        let node_rect = Rect::from_min_size(screen_pos, vec2(w, h));

        // Node background
        let is_selected = state.selected_node == Some(nid);
        let border_color = if is_selected {
            ACCENT_COLOR
        } else {
            Color32::from_rgb(0x55, 0x55, 0x55)
        };
        let border_width = if is_selected { 2.0 } else { 1.0 };

        let r = (ROUNDING * state.zoom) as u8;
        painter.rect(
            node_rect,
            CornerRadius::same(r),
            NODE_BG,
            Stroke::new(border_width, border_color),
            StrokeKind::Outside,
        );

        // Title bar
        let title_rect = Rect::from_min_size(screen_pos, vec2(w, TITLE_HEIGHT * state.zoom));
        painter.rect_filled(
            title_rect,
            CornerRadius {
                nw: r,
                ne: r,
                sw: 0,
                se: 0,
            },
            title_bar_color(node.kind),
        );
        painter.text(
            title_rect.center(),
            egui::Align2::CENTER_CENTER,
            node.kind.label(),
            egui::FontId::proportional(12.0 * state.zoom),
            Color32::BLACK,
        );

        // Pins -- use each pin's global index for layout positioning.
        for pin in &node.pins {
            let dir = pin.direction;
            let global_pin_idx = node.pins.iter().position(|p| p.id == pin.id).unwrap();
            let pp = pin_screen_pos(
                node,
                global_pin_idx,
                dir,
                state.camera_offset,
                state.zoom,
                canvas_min,
            );

            // Pin circle
            painter.circle_filled(pp, PIN_RADIUS * state.zoom, pin_color(pin.pin_type));
            painter.circle_stroke(
                pp,
                PIN_RADIUS * state.zoom,
                Stroke::new(1.0, Color32::BLACK),
            );

            // Pin label
            let label_offset = if dir == PinDirection::Input {
                vec2(12.0 * state.zoom, 0.0)
            } else {
                vec2(-12.0 * state.zoom, 0.0)
            };
            let align = if dir == PinDirection::Input {
                egui::Align2::LEFT_CENTER
            } else {
                egui::Align2::RIGHT_CENTER
            };
            painter.text(
                pp + label_offset,
                align,
                &pin.name,
                egui::FontId::proportional(10.0 * state.zoom),
                Color32::from_rgb(0xcc, 0xcc, 0xcc),
            );

            // Pin interaction area
            let pin_rect = Rect::from_center_size(
                pp,
                vec2(PIN_RADIUS * 3.0 * state.zoom, PIN_RADIUS * 3.0 * state.zoom),
            );
            let pin_resp = ui.interact(
                pin_rect,
                Id::new(("pin", nid.0, pin.id.0)),
                Sense::click_and_drag(),
            );

            if pin_resp.drag_started() && pin.direction == PinDirection::Output {
                drag_started = Some((nid, pin.id));
            }
            if pin_resp.hovered() && pin.direction == PinDirection::Input {
                ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
                if ui.input(|i| i.pointer.any_released()) {
                    drag_ended_pin = Some((nid, pin.id));
                }
            }
        }

        // Node body interaction
        let node_resp = ui.interact(
            node_rect,
            Id::new(("node", nid.0)),
            Sense::click_and_drag(),
        );
        if node_resp.clicked() {
            clicked_node = Some(nid);
        }
        if node_resp.drag_started() {
            state.dragging_node = Some(nid);
        }
        if node_resp.dragged() && state.dragging_node == Some(nid) {
            let delta = node_resp.drag_delta();
            if let Some(n) = state.graph.nodes.get_mut(&nid) {
                n.position.x += delta.x / state.zoom;
                n.position.y += delta.y / state.zoom;
            }
        }
        if node_resp.drag_stopped() && state.dragging_node == Some(nid) {
            state.dragging_node = None;
        }
    }

    // Handle node selection
    if let Some(nid) = clicked_node {
        state.selected_node = Some(nid);
    } else if response.clicked() {
        state.selected_node = None;
    }

    // Handle wire dragging
    if let Some((nid, pid)) = drag_started {
        let mouse = ui
            .input(|i| i.pointer.hover_pos())
            .unwrap_or(canvas_min);
        state.dragging_wire = Some(DragWire {
            from_node: nid,
            from_pin: pid,
            current_pos: screen_to_graph(mouse, state.camera_offset, state.zoom, canvas_min),
        });
    }

    if state.dragging_wire.is_some()
        && let Some(mouse) = ui.input(|i| i.pointer.hover_pos())
        && let Some(ref mut dw) = state.dragging_wire
    {
        dw.current_pos = screen_to_graph(mouse, state.camera_offset, state.zoom, canvas_min);
    }

    // Complete connection
    if let (Some(dw), Some((to_node, to_pin))) = (&state.dragging_wire, drag_ended_pin) {
        state
            .graph
            .connect(dw.from_node, dw.from_pin, to_node, to_pin);
    }

    if ui.input(|i| i.pointer.any_released()) {
        state.dragging_wire = None;
    }

    // ── Right-click context menu ────────────────────────────────────
    super::menus::canvas_context_menu(ui, &response, state, canvas_min);
}

// ─────────────────────────────────────────────────────────────────────
// Test helpers (expose private coordinate/bezier fns for unit tests)
// ─────────────────────────────────────────────────────────────────────

#[doc(hidden)]
pub fn __test_graph_to_screen(pos: glam::Vec2, offset: glam::Vec2, zoom: f32, canvas_min: Pos2) -> Pos2 {
    graph_to_screen(pos, offset, zoom, canvas_min)
}

#[doc(hidden)]
pub fn __test_screen_to_graph(screen: Pos2, offset: glam::Vec2, zoom: f32, canvas_min: Pos2) -> glam::Vec2 {
    screen_to_graph(screen, offset, zoom, canvas_min)
}

#[doc(hidden)]
pub fn __test_bezier_points(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, segments: usize) -> Vec<Pos2> {
    bezier_points(p0, p1, p2, p3, segments)
}

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

/// Look up the screen position of a pin given its node and pin IDs.
fn pin_pos_for_connection(
    graph: &MaterialGraph,
    node_id: NodeId,
    pin_id: PinId,
    offset: glam::Vec2,
    zoom: f32,
    canvas_min: Pos2,
) -> Option<Pos2> {
    let node = graph.nodes.get(&node_id)?;
    let pin_idx = node.pins.iter().position(|p| p.id == pin_id)?;
    let pin = &node.pins[pin_idx];
    Some(pin_screen_pos(
        node,
        pin_idx,
        pin.direction,
        offset,
        zoom,
        canvas_min,
    ))
}

/// Draw a cubic Bezier connection wire between two pin positions.
fn draw_bezier_wire(painter: &egui::Painter, from: Pos2, to: Pos2, color: Color32) {
    let dx = (to.x - from.x).abs() * 0.5;
    let cp1 = pos2(from.x + dx, from.y);
    let cp2 = pos2(to.x - dx, to.y);

    let points = bezier_points(from, cp1, cp2, to, 32);
    for w in points.windows(2) {
        painter.line_segment([w[0], w[1]], Stroke::new(2.0, color));
    }
}

/// Evaluate a cubic Bezier curve at `segments + 1` evenly spaced t values.
fn bezier_points(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, segments: usize) -> Vec<Pos2> {
    (0..=segments)
        .map(|i| {
            let t = i as f32 / segments as f32;
            let it = 1.0 - t;
            let x = it * it * it * p0.x
                + 3.0 * it * it * t * p1.x
                + 3.0 * it * t * t * p2.x
                + t * t * t * p3.x;
            let y = it * it * it * p0.y
                + 3.0 * it * it * t * p1.y
                + 3.0 * it * t * t * p2.y
                + t * t * t * p3.y;
            pos2(x, y)
        })
        .collect()
}
