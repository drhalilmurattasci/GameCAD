//! egui-based visual node graph editor for material graphs.

pub mod canvas;
pub mod menus;
pub mod state;

// Re-export public types for backwards compatibility.
pub use canvas::show;
pub use state::{DragWire, NodeEditorState};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::MaterialGraph;
    use egui::pos2;

    #[test]
    fn editor_state_creation() {
        let graph = MaterialGraph::new();
        let state = NodeEditorState::new(graph);
        assert_eq!(state.zoom, 1.0);
        assert!(state.selected_node.is_none());
    }

    #[test]
    fn graph_to_screen_identity() {
        let pos = canvas::__test_graph_to_screen(
            glam::Vec2::new(100.0, 50.0),
            glam::Vec2::ZERO,
            1.0,
            pos2(0.0, 0.0),
        );
        assert!((pos.x - 100.0).abs() < 1e-3);
        assert!((pos.y - 50.0).abs() < 1e-3);
    }

    #[test]
    fn screen_to_graph_roundtrip() {
        let offset = glam::Vec2::new(10.0, 20.0);
        let zoom = 1.5;
        let origin = pos2(0.0, 0.0);
        let original = glam::Vec2::new(100.0, 200.0);
        let screen = canvas::__test_graph_to_screen(original, offset, zoom, origin);
        let back = canvas::__test_screen_to_graph(screen, offset, zoom, origin);
        assert!((back.x - original.x).abs() < 1e-3);
        assert!((back.y - original.y).abs() < 1e-3);
    }

    #[test]
    fn bezier_endpoints() {
        let pts = canvas::__test_bezier_points(pos2(0.0, 0.0), pos2(1.0, 0.0), pos2(2.0, 0.0), pos2(3.0, 0.0), 10);
        assert_eq!(pts.len(), 11);
        assert!((pts[0].x).abs() < 1e-3);
        assert!((pts[10].x - 3.0).abs() < 1e-3);
    }
}
