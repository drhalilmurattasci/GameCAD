//! Asset metadata and sidecar `.meta.toml` file persistence.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use forge_core::id::AssetId;
use serde::{Deserialize, Serialize};

use crate::asset_type::AssetType;

/// Import settings that control how a raw file is processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSettings {
    /// Whether to generate mipmaps (textures only).
    pub generate_mipmaps: bool,
    /// Desired compression format, if any.
    pub compression: Option<String>,
    /// Scale factor applied on import (meshes).
    pub scale: f32,
    /// Whether to flip UVs vertically (textures).
    pub flip_y: bool,
}

impl Default for ImportSettings {
    fn default() -> Self {
        Self {
            generate_mipmaps: true,
            compression: None,
            scale: 1.0,
            flip_y: false,
        }
    }
}

/// Metadata describing a single asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMetadata {
    /// Unique identifier for this asset.
    pub id: AssetId,
    /// Classified type of the asset.
    pub asset_type: AssetType,
    /// Path to the source file on disk (relative to the project root).
    pub source_path: PathBuf,
    /// Import settings for this asset.
    pub import_settings: ImportSettings,
    /// Last modification time as a Unix timestamp in seconds.
    pub last_modified: u64,
}

impl AssetMetadata {
    /// Creates new metadata for a file.
    #[inline]
    pub fn new(asset_type: AssetType, source_path: PathBuf) -> Self {
        Self {
            id: AssetId::new(),
            asset_type,
            source_path,
            import_settings: ImportSettings::default(),
            last_modified: 0,
        }
    }

    /// Returns the path to the sidecar `.meta.toml` file for a given source path.
    #[inline]
    pub fn meta_path(source_path: &Path) -> PathBuf {
        let mut meta = source_path.as_os_str().to_owned();
        meta.push(".meta.toml");
        PathBuf::from(meta)
    }

    /// Saves this metadata to the sidecar file next to the source.
    pub fn save(&self) -> Result<()> {
        let meta_path = Self::meta_path(&self.source_path);
        let toml_str =
            toml::to_string_pretty(self).context("Failed to serialize asset metadata")?;
        std::fs::write(&meta_path, toml_str)
            .with_context(|| format!("Failed to write metadata to {}", meta_path.display()))?;
        Ok(())
    }

    /// Loads metadata from a sidecar `.meta.toml` file.
    pub fn load(meta_path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(meta_path)
            .with_context(|| format!("Failed to read metadata from {}", meta_path.display()))?;
        let metadata: Self =
            toml::from_str(&content).context("Failed to parse asset metadata TOML")?;
        Ok(metadata)
    }

    /// Loads metadata for a source file, if its sidecar exists.
    pub fn load_for_source(source_path: &Path) -> Result<Option<Self>> {
        let meta_path = Self::meta_path(source_path);
        if meta_path.exists() {
            Ok(Some(Self::load(&meta_path)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_path_appends_extension() {
        let src = Path::new("/assets/model.glb");
        let meta = AssetMetadata::meta_path(src);
        assert_eq!(meta, PathBuf::from("/assets/model.glb.meta.toml"));
    }

    #[test]
    fn default_import_settings() {
        let settings = ImportSettings::default();
        assert!(settings.generate_mipmaps);
        assert_eq!(settings.scale, 1.0);
        assert!(!settings.flip_y);
        assert!(settings.compression.is_none());
    }

    #[test]
    fn serde_roundtrip() {
        let meta = AssetMetadata::new(AssetType::Mesh, PathBuf::from("models/cube.glb"));
        let toml_str = toml::to_string_pretty(&meta).unwrap();
        let back: AssetMetadata = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.id, meta.id);
        assert_eq!(back.asset_type, meta.asset_type);
        assert_eq!(back.source_path, meta.source_path);
    }

    #[test]
    fn serde_roundtrip_preserves_import_settings() {
        let mut meta = AssetMetadata::new(AssetType::Texture, PathBuf::from("t.png"));
        meta.import_settings.generate_mipmaps = false;
        meta.import_settings.flip_y = true;
        meta.import_settings.scale = 2.5;
        meta.import_settings.compression = Some("bc7".to_string());
        meta.last_modified = 1234567890;

        let toml_str = toml::to_string_pretty(&meta).unwrap();
        let back: AssetMetadata = toml::from_str(&toml_str).unwrap();
        assert!(!back.import_settings.generate_mipmaps);
        assert!(back.import_settings.flip_y);
        assert!((back.import_settings.scale - 2.5).abs() < 1e-5);
        assert_eq!(back.import_settings.compression, Some("bc7".to_string()));
        assert_eq!(back.last_modified, 1234567890);
    }

    #[test]
    fn file_roundtrip_save_load() {
        let dir = std::env::temp_dir().join("forge_meta_test");
        std::fs::create_dir_all(&dir).unwrap();
        let source_path = dir.join("cube.glb");
        std::fs::write(&source_path, b"fake").unwrap();

        let mut meta = AssetMetadata::new(AssetType::Mesh, source_path.clone());
        meta.import_settings.scale = 0.01;
        meta.last_modified = 42;
        meta.save().unwrap();

        let loaded = AssetMetadata::load_for_source(&source_path)
            .unwrap()
            .expect("sidecar should exist");
        assert_eq!(loaded.id, meta.id);
        assert_eq!(loaded.asset_type, AssetType::Mesh);
        assert!((loaded.import_settings.scale - 0.01).abs() < 1e-5);
        assert_eq!(loaded.last_modified, 42);

        // Clean up.
        let _ = std::fs::remove_file(&source_path);
        let _ = std::fs::remove_file(AssetMetadata::meta_path(&source_path));
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn load_for_source_missing_returns_none() {
        let path = std::env::temp_dir().join("forge_meta_test_nonexistent.glb");
        // Ensure the meta file doesn't exist.
        let _ = std::fs::remove_file(AssetMetadata::meta_path(&path));
        let result = AssetMetadata::load_for_source(&path).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn load_invalid_meta_file_errors() {
        let dir = std::env::temp_dir().join("forge_meta_test_invalid");
        std::fs::create_dir_all(&dir).unwrap();
        let meta_path = dir.join("bad.meta.toml");
        std::fs::write(&meta_path, "this is not valid toml structure [[[").unwrap();
        assert!(AssetMetadata::load(&meta_path).is_err());
        let _ = std::fs::remove_file(&meta_path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn new_metadata_has_zero_last_modified() {
        let meta = AssetMetadata::new(AssetType::Mesh, PathBuf::from("x.glb"));
        assert_eq!(meta.last_modified, 0);
    }
}
