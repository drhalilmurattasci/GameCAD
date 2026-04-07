//! PBR material definition with TOML serialization.

use std::path::Path;

use anyhow::{Context, Result};
use forge_core::prelude::{AssetId, Color, MaterialId};
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────
// Enums
// ─────────────────────────────────────────────────────────────────────

/// A color value or a reference to a texture asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorOrTexture {
    /// An inline RGBA color.
    Color(Color),
    /// A reference to a texture asset.
    Texture(AssetId),
}

/// A scalar float value or a reference to a texture asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FloatOrTexture {
    /// An inline scalar value.
    Value(f32),
    /// A reference to a texture asset whose channel(s) provide the scalar.
    Texture(AssetId),
}

/// Emissive light configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissiveConfig {
    /// Emissive color.
    pub color: Color,
    /// Emissive intensity multiplier.
    pub strength: f32,
}

impl Default for EmissiveConfig {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            strength: 0.0,
        }
    }
}

/// How alpha (transparency) is handled.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AlphaMode {
    /// Fully opaque -- alpha channel is ignored.
    #[default]
    Opaque,
    /// Alpha-tested with the given cutoff threshold.
    Mask(f32),
    /// Alpha-blended transparency.
    Blend,
}

// ─────────────────────────────────────────────────────────────────────
// PbrMaterial
// ─────────────────────────────────────────────────────────────────────

/// A physically-based rendering material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbrMaterial {
    /// Unique identifier for this material instance.
    pub id: MaterialId,
    /// Human-readable name shown in the editor UI.
    pub name: String,
    /// Base color (albedo) -- either a flat color or a texture reference.
    pub albedo: ColorOrTexture,
    /// Optional tangent-space normal map texture.
    pub normal_map: Option<AssetId>,
    /// Metallic factor or texture (0 = dielectric, 1 = metal).
    pub metallic: FloatOrTexture,
    /// Roughness factor or texture (0 = smooth, 1 = rough).
    pub roughness: FloatOrTexture,
    /// Optional ambient-occlusion map.
    pub ao_map: Option<AssetId>,
    /// Emissive light configuration.
    pub emissive: EmissiveConfig,
    /// How transparency / alpha is handled.
    pub alpha_mode: AlphaMode,
    /// Whether the material should be rendered on both sides of a face.
    pub double_sided: bool,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            id: MaterialId::new(),
            name: String::from("Untitled Material"),
            albedo: ColorOrTexture::Color(Color::new(0.5, 0.5, 0.5, 1.0)),
            normal_map: None,
            metallic: FloatOrTexture::Value(0.0),
            roughness: FloatOrTexture::Value(0.5),
            ao_map: None,
            emissive: EmissiveConfig::default(),
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// TOML persistence
// ─────────────────────────────────────────────────────────────────────

/// Serialize a material to a TOML file.
pub fn save_material(mat: &PbrMaterial, path: &Path) -> Result<()> {
    let toml_str = toml::to_string_pretty(mat).context("Failed to serialize material to TOML")?;
    std::fs::write(path, toml_str).context("Failed to write material file")?;
    Ok(())
}

/// Deserialize a material from a TOML file.
pub fn load_material(path: &Path) -> Result<PbrMaterial> {
    let contents = std::fs::read_to_string(path).context("Failed to read material file")?;
    let mat: PbrMaterial =
        toml::from_str(&contents).context("Failed to deserialize material from TOML")?;
    Ok(mat)
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_material_is_valid() {
        let mat = PbrMaterial::default();
        assert_eq!(mat.name, "Untitled Material");
        assert!(!mat.double_sided);
    }

    #[test]
    fn toml_roundtrip() {
        let mat = PbrMaterial {
            name: "Test Mat".into(),
            albedo: ColorOrTexture::Color(Color::new(1.0, 0.0, 0.0, 1.0)),
            metallic: FloatOrTexture::Value(0.8),
            roughness: FloatOrTexture::Value(0.2),
            alpha_mode: AlphaMode::Mask(0.5),
            ..PbrMaterial::default()
        };

        let toml_str = toml::to_string_pretty(&mat).unwrap();
        let back: PbrMaterial = toml::from_str(&toml_str).unwrap();

        assert_eq!(back.name, "Test Mat");
        match back.metallic {
            FloatOrTexture::Value(v) => assert!((v - 0.8).abs() < 1e-5),
            _ => panic!("Expected Value"),
        }
    }

    #[test]
    fn save_and_load_file() {
        let dir = std::env::temp_dir().join("forge_mat_test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.material.toml");

        let mat = PbrMaterial::default();
        save_material(&mat, &path).unwrap();
        let loaded = load_material(&path).unwrap();
        assert_eq!(loaded.name, mat.name);
        assert_eq!(loaded.id, mat.id);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
