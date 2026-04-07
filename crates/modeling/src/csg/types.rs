//! CSG internal types: CsgVertex, Polygon, CsgPlane, Classification.

use glam::{Vec2, Vec3};

/// Tolerance for plane-point classification to avoid floating-point noise.
pub(crate) const EPSILON: f32 = 1e-5;

/// A vertex in the CSG working representation.
#[derive(Debug, Clone)]
pub(crate) struct CsgVertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

/// A convex polygon used during BSP clipping.
#[derive(Debug, Clone)]
pub(crate) struct Polygon {
    pub vertices: Vec<CsgVertex>,
    pub plane: CsgPlane,
}

/// An oriented plane represented as `normal . P = w`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CsgPlane {
    pub normal: Vec3,
    /// Signed distance from the origin along `normal`.
    pub w: f32,
}

impl CsgPlane {
    /// Construct a plane from three non-collinear points (CCW winding).
    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Self {
        let normal = (b - a).cross(c - a).normalize_or_zero();
        let w = normal.dot(a);
        Self { normal, w }
    }

    /// Classify a point as in front of, behind, or on this plane.
    pub fn classify_point(&self, point: Vec3) -> Classification {
        let d = self.normal.dot(point) - self.w;
        if d > EPSILON {
            Classification::Front
        } else if d < -EPSILON {
            Classification::Back
        } else {
            Classification::Coplanar
        }
    }
}

/// Result of classifying a point or polygon against a plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Classification {
    /// On the plane (within [`EPSILON`]).
    Coplanar,
    /// In front of the plane (positive half-space).
    Front,
    /// Behind the plane (negative half-space).
    Back,
    /// Polygon straddles the plane (has vertices on both sides).
    Spanning,
}

impl Polygon {
    /// Classify this polygon relative to a splitting plane.
    pub fn classify(&self, plane: &CsgPlane) -> Classification {
        let mut front = 0;
        let mut back = 0;

        for v in &self.vertices {
            match plane.classify_point(v.position) {
                Classification::Front => front += 1,
                Classification::Back => back += 1,
                _ => {}
            }
        }

        if front > 0 && back > 0 {
            Classification::Spanning
        } else if front > 0 {
            Classification::Front
        } else if back > 0 {
            Classification::Back
        } else {
            Classification::Coplanar
        }
    }

    /// Reverse winding order and flip normals (inside becomes outside).
    pub fn flip(&mut self) {
        self.vertices.reverse();
        for v in &mut self.vertices {
            v.normal = -v.normal;
        }
        self.plane.normal = -self.plane.normal;
        self.plane.w = -self.plane.w;
    }

    /// Split this polygon by a plane, pushing the front and back halves
    /// into the respective output vectors.
    pub fn split(
        &self,
        plane: &CsgPlane,
        front: &mut Vec<Polygon>,
        back: &mut Vec<Polygon>,
    ) {
        let mut f_verts = Vec::new();
        let mut b_verts = Vec::new();

        let n = self.vertices.len();
        for i in 0..n {
            let j = (i + 1) % n;
            let vi = &self.vertices[i];
            let vj = &self.vertices[j];

            let ci = plane.classify_point(vi.position);
            let cj = plane.classify_point(vj.position);

            if ci != Classification::Back {
                f_verts.push(vi.clone());
            }
            if ci != Classification::Front {
                b_verts.push(vi.clone());
            }

            if (ci == Classification::Front && cj == Classification::Back)
                || (ci == Classification::Back && cj == Classification::Front)
            {
                // Compute intersection point.
                let d_i = plane.normal.dot(vi.position) - plane.w;
                let d_j = plane.normal.dot(vj.position) - plane.w;
                let t = d_i / (d_i - d_j);
                let t = t.clamp(0.0, 1.0);

                let pos = vi.position.lerp(vj.position, t);
                let norm = vi.normal.lerp(vj.normal, t).normalize_or_zero();
                let uv = vi.uv.lerp(vj.uv, t);

                let split_v = CsgVertex {
                    position: pos,
                    normal: norm,
                    uv,
                };

                f_verts.push(split_v.clone());
                b_verts.push(split_v);
            }
        }

        if f_verts.len() >= 3 {
            let p = CsgPlane::from_points(
                f_verts[0].position,
                f_verts[1].position,
                f_verts[2].position,
            );
            front.push(Polygon {
                vertices: f_verts,
                plane: p,
            });
        }

        if b_verts.len() >= 3 {
            let p = CsgPlane::from_points(
                b_verts[0].position,
                b_verts[1].position,
                b_verts[2].position,
            );
            back.push(Polygon {
                vertices: b_verts,
                plane: p,
            });
        }
    }
}
