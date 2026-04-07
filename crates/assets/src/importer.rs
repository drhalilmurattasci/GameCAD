//! File importing -- detects asset types and registers them in the database.

use std::path::Path;

use anyhow::{bail, Result};
use forge_core::id::AssetId;
use tracing::info;

use crate::asset_type::AssetType;
use crate::database::AssetDatabase;
use crate::metadata::AssetMetadata;

/// The result of importing a single file.
#[derive(Debug)]
pub struct ImportResult {
    /// Identifier assigned to the imported asset.
    pub asset_id: AssetId,
    /// Detected asset type.
    pub asset_type: AssetType,
    /// Full metadata record.
    pub metadata: AssetMetadata,
}

/// Imports a file into the asset database.
///
/// The file's extension is used to detect its [`AssetType`]. If a sidecar
/// `.meta.toml` already exists it is loaded; otherwise new metadata is created
/// and persisted.
pub fn import_file(path: &Path, database: &mut AssetDatabase) -> Result<ImportResult> {
    let Some(asset_type) = AssetType::from_path(path) else {
        bail!(
            "Unsupported file type: {}",
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("<none>")
        );
    };

    // Reuse existing metadata when available.
    let metadata = match AssetMetadata::load_for_source(path)? {
        Some(existing) => existing,
        None => {
            let mut meta = AssetMetadata::new(asset_type, path.to_path_buf());
            if let Ok(fs_meta) = std::fs::metadata(path)
                && let Ok(modified) = fs_meta.modified()
            {
                meta.last_modified = modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
            }
            // Persist the new sidecar.
            if let Err(e) = meta.save() {
                tracing::warn!("Could not save metadata sidecar: {}", e);
            }
            meta
        }
    };

    let asset_id = metadata.id;

    info!(
        "Imported {} as {:?} ({})",
        path.display(),
        asset_type,
        asset_id
    );

    database.register_asset(metadata.clone());

    Ok(ImportResult {
        asset_id,
        asset_type,
        metadata,
    })
}

/// Returns a list of all file extensions the importer can handle.
pub fn supported_extensions() -> &'static [&'static str] {
    &[
        // Mesh
        "gltf", "glb", "obj", "fbx", "stl", "ply",
        // Texture
        "png", "jpg", "jpeg", "bmp", "tga", "webp", "dds", "ktx2",
        // Material
        "mat", "material",
        // Scene
        "scene", "scn",
        // HDRI
        "hdr", "exr",
        // Animation
        "anim",
        // Audio
        "wav", "ogg", "mp3", "flac",
        // Script
        "lua", "luau",
        // Font
        "ttf", "otf", "woff", "woff2",
        // Shader
        "wgsl", "glsl", "vert", "frag", "comp", "hlsl",
        // Theme
        "theme",
        // Prefab
        "prefab",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_extensions_not_empty() {
        assert!(!supported_extensions().is_empty());
    }

    #[test]
    fn supported_extensions_include_common_types() {
        let exts = supported_extensions();
        assert!(exts.contains(&"glb"));
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"lua"));
        assert!(exts.contains(&"wgsl"));
    }

    #[test]
    fn all_supported_extensions_are_recognized() {
        for ext in supported_extensions() {
            assert!(
                AssetType::from_extension(ext).is_some(),
                "Extension '{ext}' is listed as supported but not recognized by AssetType"
            );
        }
    }

    #[test]
    fn import_unsupported_file_errors() {
        let dir = std::env::temp_dir().join("forge_import_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("readme.txt");
        std::fs::write(&path, b"hello").unwrap();

        let mut db = AssetDatabase::new();
        let result = import_file(&path, &mut db);
        assert!(result.is_err());

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn import_file_registers_in_database() {
        let dir = std::env::temp_dir().join("forge_import_test_register");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.lua");
        std::fs::write(&path, b"print('hello')").unwrap();

        let mut db = AssetDatabase::new();
        let result = import_file(&path, &mut db).unwrap();
        assert_eq!(result.asset_type, AssetType::Script);
        assert!(db.get_by_id(result.asset_id).is_some());

        // Clean up.
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(crate::metadata::AssetMetadata::meta_path(&path));
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn import_file_no_extension_errors() {
        let dir = std::env::temp_dir().join("forge_import_test_noext");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("Makefile");
        std::fs::write(&path, b"all:").unwrap();

        let mut db = AssetDatabase::new();
        assert!(import_file(&path, &mut db).is_err());

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }
}
