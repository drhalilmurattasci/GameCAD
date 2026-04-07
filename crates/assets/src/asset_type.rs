//! Asset type classification and file extension mapping.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// The kind of asset a file represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    Mesh,
    Texture,
    Material,
    Scene,
    Hdri,
    Animation,
    Audio,
    Script,
    Font,
    Shader,
    Theme,
    Prefab,
}

impl AssetType {
    /// Attempts to detect the asset type from a file extension.
    ///
    /// Returns `None` if the extension is not recognized.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            // Mesh formats
            "gltf" | "glb" | "obj" | "fbx" | "stl" | "ply" => Some(Self::Mesh),
            // Texture / image formats
            "png" | "jpg" | "jpeg" | "bmp" | "tga" | "webp" | "dds" | "ktx2" => {
                Some(Self::Texture)
            }
            // Material definitions
            "mat" | "material" => Some(Self::Material),
            // Scene files
            "scene" | "scn" => Some(Self::Scene),
            // HDR environment maps
            "hdr" | "exr" => Some(Self::Hdri),
            // Animations
            "anim" => Some(Self::Animation),
            // Audio
            "wav" | "ogg" | "mp3" | "flac" => Some(Self::Audio),
            // Scripts
            "lua" | "luau" => Some(Self::Script),
            // Fonts
            "ttf" | "otf" | "woff" | "woff2" => Some(Self::Font),
            // Shaders
            "wgsl" | "glsl" | "vert" | "frag" | "comp" | "hlsl" => Some(Self::Shader),
            // Themes
            "theme" => Some(Self::Theme),
            // Prefabs
            "prefab" => Some(Self::Prefab),
            _ => None,
        }
    }

    /// Detects the asset type from a file path by inspecting its extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Returns a single-character icon suitable for UI display.
    pub fn icon_char(&self) -> char {
        match self {
            Self::Mesh => '\u{25B2}',      // triangle
            Self::Texture => '\u{1F5BC}',  // framed picture
            Self::Material => '\u{25CF}',  // filled circle
            Self::Scene => '\u{1F3AC}',    // clapper board
            Self::Hdri => '\u{2600}',      // sun
            Self::Animation => '\u{23F5}', // play button
            Self::Audio => '\u{266B}',     // beamed eighth notes
            Self::Script => '\u{1F4DC}',   // scroll
            Self::Font => '\u{0041}',      // letter A
            Self::Shader => '\u{2726}',    // four pointed star
            Self::Theme => '\u{1F3A8}',    // palette
            Self::Prefab => '\u{1F4E6}',   // package
        }
    }

    /// Human-readable label for the asset type.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Mesh => "Mesh",
            Self::Texture => "Texture",
            Self::Material => "Material",
            Self::Scene => "Scene",
            Self::Hdri => "HDRI",
            Self::Animation => "Animation",
            Self::Audio => "Audio",
            Self::Script => "Script",
            Self::Font => "Font",
            Self::Shader => "Shader",
            Self::Theme => "Theme",
            Self::Prefab => "Prefab",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_mesh_extension() {
        assert_eq!(AssetType::from_extension("glb"), Some(AssetType::Mesh));
        assert_eq!(AssetType::from_extension("GLTF"), Some(AssetType::Mesh));
    }

    #[test]
    fn detect_texture_extension() {
        assert_eq!(AssetType::from_extension("png"), Some(AssetType::Texture));
        assert_eq!(AssetType::from_extension("jpg"), Some(AssetType::Texture));
    }

    #[test]
    fn unknown_extension_returns_none() {
        assert_eq!(AssetType::from_extension("xyz"), None);
    }

    #[test]
    fn from_path_works() {
        let path = Path::new("/assets/model.glb");
        assert_eq!(AssetType::from_path(path), Some(AssetType::Mesh));
    }
}
