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
}
