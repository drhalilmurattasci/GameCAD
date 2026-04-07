//! Dynamically-typed property values for the inspector.

use forge_core::id::AssetId;
use forge_core::math::{Color, Vec2, Vec3, Vec4};
use serde::{Deserialize, Serialize};

/// A single property value that can be displayed and edited in the inspector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PropertyValue {
    /// A single-precision float.
    Float(f32),
    /// A 2-component vector.
    Vec2(Vec2),
    /// A 3-component vector.
    Vec3(Vec3),
    /// A 4-component vector.
    Vec4(Vec4),
    /// A linear RGBA color.
    Color(Color),
    /// A boolean toggle.
    Bool(bool),
    /// A text string.
    String(String),
    /// A signed 32-bit integer.
    Int(i32),
    /// An enumeration with a list of options and a selected index.
    Enum {
        options: Vec<String>,
        selected: usize,
    },
    /// A reference to an asset by ID.
    AssetRef(AssetId),
}

impl PropertyValue {
    /// Returns a human-readable type label.
    pub fn type_label(&self) -> &'static str {
        match self {
            Self::Float(_) => "Float",
            Self::Vec2(_) => "Vec2",
            Self::Vec3(_) => "Vec3",
            Self::Vec4(_) => "Vec4",
            Self::Color(_) => "Color",
            Self::Bool(_) => "Bool",
            Self::String(_) => "String",
            Self::Int(_) => "Int",
            Self::Enum { .. } => "Enum",
            Self::AssetRef(_) => "AssetRef",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_labels() {
        assert_eq!(PropertyValue::Float(1.0).type_label(), "Float");
        assert_eq!(PropertyValue::Bool(true).type_label(), "Bool");
        assert_eq!(
            PropertyValue::Enum {
                options: vec!["A".into()],
                selected: 0,
            }
            .type_label(),
            "Enum"
        );
    }
}
