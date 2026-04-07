//! Right-click context menus: add/delete/duplicate node menus.

use egui::{pos2, vec2, Id, Pos2, Rect, Sense, Ui};

use crate::graph::{MaterialNode, NodeId, NodeKind};

use super::state::NodeEditorState;

// ─────────────────────────────────────────────────────────────────────
// Constants (duplicated from canvas for node_height calculation)
// ─────────────────────────────────────────────────────────────────────

const NODE_WIDTH: f32 = 180.0;
const TITLE_HEIGHT: f32 = 28.0;
const PIN_ROW_HEIGHT: f32 = 22.0;

/// Compute the visual height of a node based on its pin count.
fn node_height(node: &MaterialNode) -> f32 {
    TITLE_HEIGHT + node.pins.len() as f32 * PIN_ROW_HEIGHT + 4.0
}

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

// ─────────────────────────────────────────────────────────────────────
// Context menus
// ─────────────────────────────────────────────────────────────────────

/// Canvas-level right-click context menu and node context menu.
pub(crate) fn canvas_context_menu(
    ui: &mut Ui,
    response: &egui::Response,
    state: &mut NodeEditorState,
    canvas_min: Pos2,
) {
    response.context_menu(|ui| {
        ui.menu_button("Add Node", |ui| {
            for &kind in NodeKind::ALL {
                if ui.button(kind.label()).clicked() {
                    let mouse = ui
                        .input(|i| i.pointer.hover_pos())
                        .unwrap_or(canvas_min);
                    let pos = screen_to_graph(mouse, state.camera_offset, state.zoom, canvas_min);
                    let node = MaterialNode::new(kind, pos);
                    state.graph.add_node(node);
                    ui.close_menu();
                }
            }
        });
    });

    // Node context menu (right-click on selected node)
    if let Some(sel_id) = state.selected_node
        && state.graph.nodes.contains_key(&sel_id)
    {
            let node = &state.graph.nodes[&sel_id];
            let sp = graph_to_screen(node.position, state.camera_offset, state.zoom, canvas_min);
            let h = node_height(node) * state.zoom;
            let node_rect = Rect::from_min_size(sp, vec2(NODE_WIDTH * state.zoom, h));
            let node_resp = ui.interact(
                node_rect,
                Id::new(("node_ctx", sel_id.0)),
                Sense::click(),
            );
            node_resp.context_menu(|ui| {
                if ui.button("Delete").clicked() {
                    state.graph.remove_node(&sel_id);
                    state.selected_node = None;
                    ui.close_menu();
                }
                if ui.button("Duplicate").clicked() {
                    if let Some(orig) = state.graph.nodes.get(&sel_id).cloned() {
                        let mut dup = MaterialNode::new(
                            orig.kind,
                            orig.position + glam::Vec2::new(30.0, 30.0),
                        );
                        dup.id = NodeId::new();
                        state.graph.add_node(dup);
                    }
                    ui.close_menu();
                }
                if ui.button("Disconnect All").clicked() {
                    state
                        .graph
                        .connections
                        .retain(|c| c.from_node != sel_id && c.to_node != sel_id);
                    ui.close_menu();
                }
            });
    }
}
