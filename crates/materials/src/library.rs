//! Material library -- stores, retrieves, and searches [`PbrMaterial`]s.

use std::collections::HashMap;
use std::path::Path;

use anyhow::Result;
use forge_core::prelude::{Color, MaterialId};
use tracing::{info, warn};

use crate::material::{
    load_material, AlphaMode, ColorOrTexture, EmissiveConfig, FloatOrTexture, PbrMaterial,
};

// ─────────────────────────────────────────────────────────────────────
// MaterialLibrary
// ─────────────────────────────────────────────────────────────────────

/// An in-memory collection of PBR materials keyed by [`MaterialId`].
#[derive(Debug, Clone)]
pub struct MaterialLibrary {
    materials: HashMap<MaterialId, PbrMaterial>,
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        let mut lib = Self {
            materials: HashMap::new(),
        };
        lib.add_defaults();
        lib
    }
}

impl MaterialLibrary {
    /// Creates an empty library (no default materials).
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    /// Inserts a material, returning the previous one if the id was already present.
    pub fn add(&mut self, mat: PbrMaterial) -> Option<PbrMaterial> {
        self.materials.insert(mat.id, mat)
    }

    /// Removes a material by id, returning it if found.
    pub fn remove(&mut self, id: &MaterialId) -> Option<PbrMaterial> {
        self.materials.remove(id)
    }

    /// Returns a reference to the material with the given id.
    pub fn get(&self, id: &MaterialId) -> Option<&PbrMaterial> {
        self.materials.get(id)
    }

    /// Returns a mutable reference to the material with the given id.
    pub fn get_mut(&mut self, id: &MaterialId) -> Option<&mut PbrMaterial> {
        self.materials.get_mut(id)
    }

    /// Returns an iterator over all (id, material) pairs.
    pub fn list(&self) -> impl Iterator<Item = (&MaterialId, &PbrMaterial)> {
        self.materials.iter()
    }

    /// Case-insensitive substring search across material names.
    pub fn search(&self, query: &str) -> Vec<&PbrMaterial> {
        let q = query.to_lowercase();
        self.materials
            .values()
            .filter(|m| m.name.to_lowercase().contains(&q))
            .collect()
    }

    /// Scan a directory for `*.material.toml` files and load them.
    pub fn load_from_directory(&mut self, path: &Path) -> Result<usize> {
        let mut count = 0usize;
        if !path.is_dir() {
            anyhow::bail!("{} is not a directory", path.display());
        }
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();
            if file_path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".material.toml"))
            {
                match load_material(&file_path) {
                    Ok(mat) => {
                        info!("Loaded material '{}' from {}", mat.name, file_path.display());
                        self.add(mat);
                        count += 1;
                    }
                    Err(e) => {
                        warn!("Failed to load {}: {e:#}", file_path.display());
                    }
                }
            }
        }
        Ok(count)
    }

    // ── Default materials ───────────────────────────────────────────

    /// Populate the library with a set of built-in starter materials.
    fn add_defaults(&mut self) {
        self.add(Self::default_gray());
        self.add(Self::red_plastic());
        self.add(Self::gold_metal());
        self.add(Self::glass());
    }

    /// Mid-gray dielectric -- a safe default for unassigned geometry.
    fn default_gray() -> PbrMaterial {
        PbrMaterial {
            name: "Default Gray".into(),
            albedo: ColorOrTexture::Color(Color::new(0.5, 0.5, 0.5, 1.0)),
            metallic: FloatOrTexture::Value(0.0),
            roughness: FloatOrTexture::Value(0.5),
            ..PbrMaterial::default()
        }
    }

    /// A shiny red dielectric (plastic-like).
    fn red_plastic() -> PbrMaterial {
        PbrMaterial {
            name: "Red Plastic".into(),
            albedo: ColorOrTexture::Color(Color::new(0.8, 0.05, 0.05, 1.0)),
            metallic: FloatOrTexture::Value(0.0),
            roughness: FloatOrTexture::Value(0.4),
            ..PbrMaterial::default()
        }
    }

    /// A polished gold metallic material.
    fn gold_metal() -> PbrMaterial {
        PbrMaterial {
            name: "Gold Metal".into(),
            albedo: ColorOrTexture::Color(Color::new(1.0, 0.76, 0.33, 1.0)),
            metallic: FloatOrTexture::Value(1.0),
            roughness: FloatOrTexture::Value(0.3),
            ..PbrMaterial::default()
        }
    }

    /// A transparent glass-like material using alpha blending.
    fn glass() -> PbrMaterial {
        PbrMaterial {
            name: "Glass".into(),
            albedo: ColorOrTexture::Color(Color::new(0.95, 0.95, 0.95, 0.3)),
            metallic: FloatOrTexture::Value(0.0),
            roughness: FloatOrTexture::Value(0.05),
            alpha_mode: AlphaMode::Blend,
            emissive: EmissiveConfig::default(),
            ..PbrMaterial::default()
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::save_material;

    #[test]
    fn default_library_has_four_materials() {
        let lib = MaterialLibrary::default();
        assert_eq!(lib.list().count(), 4);
    }

    #[test]
    fn add_and_get() {
        let mut lib = MaterialLibrary::new();
        let mat = PbrMaterial::default();
        let id = mat.id;
        lib.add(mat);
        assert!(lib.get(&id).is_some());
    }

    #[test]
    fn remove() {
        let mut lib = MaterialLibrary::default();
        let ids: Vec<MaterialId> = lib.list().map(|(id, _)| *id).collect();
        let removed = lib.remove(&ids[0]);
        assert!(removed.is_some());
        assert_eq!(lib.list().count(), 3);
    }

    #[test]
    fn search_by_name() {
        let lib = MaterialLibrary::default();
        let results = lib.search("gold");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Gold Metal");
    }

    #[test]
    fn search_case_insensitive() {
        let lib = MaterialLibrary::default();
        assert_eq!(lib.search("GLASS").len(), 1);
    }

    #[test]
    fn load_from_directory() {
        let dir = std::env::temp_dir().join("forge_matlib_test");
        let _ = std::fs::create_dir_all(&dir);

        let mat = PbrMaterial {
            name: "Loaded Mat".into(),
            ..PbrMaterial::default()
        };
        save_material(&mat, &dir.join("loaded.material.toml")).unwrap();

        let mut lib = MaterialLibrary::new();
        let count = lib.load_from_directory(&dir).unwrap();
        assert_eq!(count, 1);
        assert_eq!(lib.search("Loaded").len(), 1);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
