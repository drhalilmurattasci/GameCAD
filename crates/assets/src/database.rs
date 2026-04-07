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
    #[inline]
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
                .is_some_and(|n| n.ends_with(".meta.toml"))
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
                    if let Ok(fs_meta) = std::fs::metadata(path)
                        && let Ok(modified) = fs_meta.modified()
                    {
                        meta.last_modified = modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();
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
    #[inline]
    pub fn get_by_id(&self, id: AssetId) -> Option<&AssetMetadata> {
        self.assets.get(&id)
    }

    /// Looks up an asset by its source path.
    #[inline]
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
    #[inline]
    pub fn len(&self) -> usize {
        self.assets.len()
    }

    /// Returns `true` if no assets are registered.
    #[inline]
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

    #[test]
    fn search_case_insensitive() {
        let mut db = AssetDatabase::new();
        db.register_asset(AssetMetadata::new(
            AssetType::Mesh,
            PathBuf::from("models/BigSword.glb"),
        ));

        let results = db.search("bigsword");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_by_type_label() {
        let mut db = AssetDatabase::new();
        db.register_asset(AssetMetadata::new(
            AssetType::Texture,
            PathBuf::from("a.png"),
        ));

        let results = db.search("texture");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_empty_query_returns_all() {
        let mut db = AssetDatabase::new();
        db.register_asset(AssetMetadata::new(
            AssetType::Mesh,
            PathBuf::from("a.glb"),
        ));
        db.register_asset(AssetMetadata::new(
            AssetType::Texture,
            PathBuf::from("b.png"),
        ));

        let results = db.search("");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_no_results() {
        let mut db = AssetDatabase::new();
        db.register_asset(AssetMetadata::new(
            AssetType::Mesh,
            PathBuf::from("a.glb"),
        ));

        let results = db.search("zzzzz_no_match");
        assert!(results.is_empty());
    }

    #[test]
    fn remove_nonexistent_returns_none() {
        let mut db = AssetDatabase::new();
        assert!(db.remove_asset(AssetId::new()).is_none());
    }

    #[test]
    fn remove_cleans_path_index() {
        let mut db = AssetDatabase::new();
        let meta = AssetMetadata::new(AssetType::Mesh, PathBuf::from("a.glb"));
        let id = meta.id;
        let path = meta.source_path.clone();
        db.register_asset(meta);
        db.remove_asset(id);
        assert!(db.get_by_path(&path).is_none());
        assert!(db.path_index.is_empty());
    }

    #[test]
    fn register_duplicate_id_overwrites() {
        let mut db = AssetDatabase::new();
        let meta1 = AssetMetadata::new(AssetType::Mesh, PathBuf::from("a.glb"));
        let id = meta1.id;
        db.register_asset(meta1);

        // Create a second metadata with the same ID but different path.
        let mut meta2 = AssetMetadata::new(AssetType::Texture, PathBuf::from("b.png"));
        meta2.id = id;
        db.register_asset(meta2);

        // Should overwrite: only 1 asset with that ID, but the old path index entry
        // for "a.glb" becomes stale.
        assert_eq!(db.assets.len(), 1);
        assert_eq!(db.get_by_id(id).unwrap().asset_type, AssetType::Texture);
    }

    #[test]
    fn new_database_is_empty() {
        let db = AssetDatabase::new();
        assert!(db.is_empty());
        assert_eq!(db.len(), 0);
    }

    #[test]
    fn find_by_type_empty_db() {
        let db = AssetDatabase::new();
        assert!(db.find_by_type(AssetType::Mesh).is_empty());
    }

    #[test]
    fn scan_empty_directory() {
        let dir = std::env::temp_dir().join("forge_asset_test_empty_scan");
        std::fs::create_dir_all(&dir).unwrap();
        // Make sure it's empty.
        for entry in std::fs::read_dir(&dir).unwrap() {
            let _ = std::fs::remove_file(entry.unwrap().path());
        }

        let mut db = AssetDatabase::new();
        let count = db.scan_directory(&dir).unwrap();
        assert_eq!(count, 0);
        assert!(db.is_empty());

        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn scan_directory_finds_assets() {
        let dir = std::env::temp_dir().join("forge_asset_test_scan");
        std::fs::create_dir_all(&dir).unwrap();
        // Create a fake .glb file.
        std::fs::write(dir.join("test.glb"), b"fake").unwrap();
        // Create a file with unknown extension -- should be skipped.
        std::fs::write(dir.join("readme.txt"), b"hi").unwrap();

        let mut db = AssetDatabase::new();
        let count = db.scan_directory(&dir).unwrap();
        assert_eq!(count, 1);
        assert_eq!(db.len(), 1);

        // Clean up.
        let _ = std::fs::remove_file(dir.join("test.glb"));
        let _ = std::fs::remove_file(dir.join("test.glb.meta.toml"));
        let _ = std::fs::remove_file(dir.join("readme.txt"));
        let _ = std::fs::remove_dir(&dir);
    }
}
