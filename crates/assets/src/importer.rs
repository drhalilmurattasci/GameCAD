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
            if let Ok(fs_meta) = std::fs::metadata(path) {
                if let Ok(modified) = fs_meta.modified() {
                    meta.last_modified = modified
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                }
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
}
