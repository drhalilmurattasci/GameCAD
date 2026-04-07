//! BSP tree for polygon clipping.

use super::types::{Classification, CsgPlane, Polygon};

/// A node in the BSP tree used for polygon clipping.
#[derive(Debug)]
pub(crate) struct BspNode {
    /// The splitting plane for this node (`None` for empty leaf nodes).
    pub plane: Option<CsgPlane>,
    /// Subtree containing polygons in front of the splitting plane.
    pub front: Option<Box<BspNode>>,
    /// Subtree containing polygons behind the splitting plane.
    pub back: Option<Box<BspNode>>,
    /// Coplanar polygons that lie on this node's splitting plane.
    pub polygons: Vec<Polygon>,
}

impl BspNode {
    pub fn new() -> Self {
        Self {
            plane: None,
            front: None,
            back: None,
            polygons: Vec::new(),
        }
    }

    pub fn build(polygons: Vec<Polygon>) -> Self {
        let mut node = BspNode::new();
        if polygons.is_empty() {
            return node;
        }

        // Use the first polygon's plane as the splitting plane.
        node.plane = Some(polygons[0].plane);
        let plane = polygons[0].plane;

        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();

        for poly in &polygons {
            match poly.classify(&plane) {
                Classification::Coplanar => {
                    // Coplanar polygons go with this node.
                    node.polygons.push(poly.clone());
                }
                Classification::Front => {
                    front_polys.push(poly.clone());
                }
                Classification::Back => {
                    back_polys.push(poly.clone());
                }
                Classification::Spanning => {
                    poly.split(&plane, &mut front_polys, &mut back_polys);
                }
            }
        }

        if !front_polys.is_empty() {
            node.front = Some(Box::new(BspNode::build(front_polys)));
        }
        if !back_polys.is_empty() {
            node.back = Some(Box::new(BspNode::build(back_polys)));
        }

        node
    }
}
