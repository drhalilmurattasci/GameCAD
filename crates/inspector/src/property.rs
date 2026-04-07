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
    #[inline]
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

    #[test]
    fn all_type_labels_non_empty() {
        let values: Vec<PropertyValue> = vec![
            PropertyValue::Float(0.0),
            PropertyValue::Vec2(Vec2::ZERO),
            PropertyValue::Vec3(Vec3::ZERO),
            PropertyValue::Vec4(Vec4::ZERO),
            PropertyValue::Color(Color::WHITE),
            PropertyValue::Bool(false),
            PropertyValue::String(String::new()),
            PropertyValue::Int(0),
            PropertyValue::Enum {
                options: vec![],
                selected: 0,
            },
            PropertyValue::AssetRef(AssetId::NIL),
        ];
        for v in &values {
            assert!(
                !v.type_label().is_empty(),
                "Empty label for {:?}",
                v.type_label()
            );
        }
    }

    #[test]
    fn serde_roundtrip_float() {
        let val = PropertyValue::Float(3.14);
        let json = serde_json::to_string(&val).unwrap();
        let back: PropertyValue = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, PropertyValue::Float(f) if (f - 3.14).abs() < 1e-5));
    }

    #[test]
    fn serde_roundtrip_string() {
        let val = PropertyValue::String("hello world".into());
        let json = serde_json::to_string(&val).unwrap();
        let back: PropertyValue = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, PropertyValue::String(s) if s == "hello world"));
    }

    #[test]
    fn serde_roundtrip_enum() {
        let val = PropertyValue::Enum {
            options: vec!["A".into(), "B".into(), "C".into()],
            selected: 2,
        };
        let json = serde_json::to_string(&val).unwrap();
        let back: PropertyValue = serde_json::from_str(&json).unwrap();
        match back {
            PropertyValue::Enum { options, selected } => {
                assert_eq!(options.len(), 3);
                assert_eq!(selected, 2);
            }
            _ => panic!("Expected Enum"),
        }
    }

    #[test]
    fn serde_roundtrip_vec3() {
        let val = PropertyValue::Vec3(Vec3::new(1.0, 2.0, 3.0));
        let json = serde_json::to_string(&val).unwrap();
        let back: PropertyValue = serde_json::from_str(&json).unwrap();
        match back {
            PropertyValue::Vec3(v) => {
                assert!((v.x - 1.0).abs() < 1e-5);
                assert!((v.y - 2.0).abs() < 1e-5);
                assert!((v.z - 3.0).abs() < 1e-5);
            }
            _ => panic!("Expected Vec3"),
        }
    }

    #[test]
    fn serde_roundtrip_color() {
        let val = PropertyValue::Color(Color::new(0.5, 0.6, 0.7, 0.8));
        let json = serde_json::to_string(&val).unwrap();
        let back: PropertyValue = serde_json::from_str(&json).unwrap();
        match back {
            PropertyValue::Color(c) => {
                assert!((c.r - 0.5).abs() < 1e-5);
                assert!((c.a - 0.8).abs() < 1e-5);
            }
            _ => panic!("Expected Color"),
        }
    }

    #[test]
    fn serde_roundtrip_asset_ref() {
        let id = AssetId::new();
        let val = PropertyValue::AssetRef(id);
        let json = serde_json::to_string(&val).unwrap();
        let back: PropertyValue = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, PropertyValue::AssetRef(back_id) if back_id == id));
    }

    #[test]
    fn clone_preserves_value() {
        let val = PropertyValue::Int(42);
        let cloned = val.clone();
        assert!(matches!(cloned, PropertyValue::Int(42)));
    }
}
