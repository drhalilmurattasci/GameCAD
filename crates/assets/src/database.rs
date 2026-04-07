//! Asset database for scanning, indexing, and querying assets.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use forge_core::id::AssetId;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use crate::asset_type::AssetType;
use crate::metadata::AssetMetadata;

/// An indexed collection of all known assets in a project.
#[derive(Debug, Default)]
pub struct AssetDatabase {
    /// Map from asset ID to its metadata.
    pub assets: HashMap<AssetId, AssetMetadata>,
    /// Reverse index from canonical source path to asset ID.
    pub path_index: HashMap<PathBuf, AssetId>,
}

impl AssetDatabase {
    /// Creates an empty database.
    pub fn new() -> Self {
        Self::default()
    }

    /// Recursively scans a directory, registering every recognized asset.
    ///
    /// Existing `.meta.toml` sidecar files are loaded when present; otherwise
    /// new metadata is created.
    pub fn scan_directory(&mut self, root: &Path) -> Result<usize> {
        let mut count = 0usize;

        for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Skip sidecar files themselves.
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map_or(false, |n| n.ends_with(".meta.toml"))
            {
                continue;
            }

            // Only index files with a known asset type.
            let Some(asset_type) = AssetType::from_path(path) else {
                continue;
            };

            // Try loading existing sidecar metadata.
            let metadata = match AssetMetadata::load_for_source(path) {
                Ok(Some(existing)) => {
                    debug!("Loaded existing metadata for {}", path.display());
                    existing
                }
                _ => {
                    debug!("Creating new metadata for {}", path.display());
                    let mut meta = AssetMetadata::new(asset_type, path.to_path_buf());
                    // Try to read last modified time.
                    if let Ok(fs_meta) = std::fs::metadata(path) {
                        if let Ok(modified) = fs_meta.modified() {
                            meta.last_modified = modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                        }
                    }
                    meta
                }
            };

            self.register_asset(metadata);
            count += 1;
        }

        info!("Scanned {} assets from {}", count, root.display());
        Ok(count)
    }

    /// Registers an asset in the database, updating the path index.
    pub fn register_asset(&mut self, metadata: AssetMetadata) {
        let id = metadata.id;
        let path = metadata.source_path.clone();
        self.path_index.insert(path, id);
        self.assets.insert(id, metadata);
    }

    /// Removes an asset by its ID. Returns the removed metadata, if any.
    pub fn remove_asset(&mut self, id: AssetId) -> Option<AssetMetadata> {
        if let Some(meta) = self.assets.remove(&id) {
            self.path_index.remove(&meta.source_path);
            Some(meta)
        } else {
            warn!("Attempted to remove unknown asset {:?}", id);
            None
        }
    }

    /// Looks up an asset by its ID.
    pub fn get_by_id(&self, id: AssetId) -> Option<&AssetMetadata> {
        self.assets.get(&id)
    }

    /// Looks up an asset by its source path.
    pub fn get_by_path(&self, path: &Path) -> Option<&AssetMetadata> {
        self.path_index
            .get(path)
            .and_then(|id| self.assets.get(id))
    }

    /// Returns all assets of a given type.
    pub fn find_by_type(&self, asset_type: AssetType) -> Vec<&AssetMetadata> {
        self.assets
            .values()
            .filter(|m| m.asset_type == asset_type)
            .collect()
    }

    /// Searches assets whose source path or type label contains `query` (case-insensitive).
    pub fn search(&self, query: &str) -> Vec<&AssetMetadata> {
        let query_lower = query.to_ascii_lowercase();
        self.assets
            .values()
            .filter(|m| {
                let path_str = m.source_path.to_string_lossy().to_ascii_lowercase();
                let type_label = m.asset_type.label().to_ascii_lowercase();
                path_str.contains(&query_lower) || type_label.contains(&query_lower)
            })
            .collect()
    }

    /// Returns the total number of registered assets.
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Returns `true` if no assets are registered.
    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut db = AssetDatabase::new();
        let meta = AssetMetadata::new(AssetType::Mesh, PathBuf::from("models/cube.glb"));
        let id = meta.id;

        db.register_asset(meta);
        assert_eq!(db.len(), 1);
        assert!(db.get_by_id(id).is_some());
        assert!(db.get_by_path(Path::new("models/cube.glb")).is_some());
    }

    #[test]
    fn remove_asset() {
        let mut db = AssetDatabase::new();
        let meta = AssetMetadata::new(AssetType::Texture, PathBuf::from("textures/wall.png"));
        let id = meta.id;

        db.register_asset(meta);
        assert!(db.remove_asset(id).is_some());
        assert!(db.get_by_id(id).is_none());
        assert!(db.is_empty());
    }

    #[test]
    fn find_by_type_filters_correctly() {
        let mut db = AssetDatabase::new();
        db.register_asset(AssetMetadata::new(
            AssetType::Mesh,
            PathBuf::from("a.glb"),
        ));
        db.register_asset(AssetMetadata::new(
            AssetType::Texture,
            PathBuf::from("b.png"),
        ));
        db.register_asset(AssetMetadata::new(
            AssetType::Mesh,
            PathBuf::from("c.glb"),
        ));

        let meshes = db.find_by_type(AssetType::Mesh);
        assert_eq!(meshes.len(), 2);
    }

    #[test]
    fn search_by_path() {
        let mut db = AssetDatabase::new();
        db.register_asset(AssetMetadata::new(
            AssetType::Mesh,
            PathBuf::from("models/hero_sword.glb"),
        ));
        db.register_asset(AssetMetadata::new(
            AssetType::Texture,
            PathBuf::from("textures/grass.png"),
        ));

        let results = db.search("sword");
        assert_eq!(results.len(), 1);
    }
}
