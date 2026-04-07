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
    #[inline]
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
    #[inline]
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Returns a single-character icon suitable for UI display.
    #[inline]
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
    #[inline]
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
    fn detect_mesh_extensions() {
        for ext in &["gltf", "glb", "obj", "fbx", "stl", "ply"] {
            assert_eq!(
                AssetType::from_extension(ext),
                Some(AssetType::Mesh),
                "Failed for extension: {ext}"
            );
        }
    }

    #[test]
    fn detect_mesh_case_insensitive() {
        assert_eq!(AssetType::from_extension("GLTF"), Some(AssetType::Mesh));
        assert_eq!(AssetType::from_extension("Glb"), Some(AssetType::Mesh));
    }

    #[test]
    fn detect_texture_extensions() {
        for ext in &["png", "jpg", "jpeg", "bmp", "tga", "webp", "dds", "ktx2"] {
            assert_eq!(
                AssetType::from_extension(ext),
                Some(AssetType::Texture),
                "Failed for extension: {ext}"
            );
        }
    }

    #[test]
    fn detect_material_extensions() {
        assert_eq!(AssetType::from_extension("mat"), Some(AssetType::Material));
        assert_eq!(
            AssetType::from_extension("material"),
            Some(AssetType::Material)
        );
    }

    #[test]
    fn detect_scene_extensions() {
        assert_eq!(AssetType::from_extension("scene"), Some(AssetType::Scene));
        assert_eq!(AssetType::from_extension("scn"), Some(AssetType::Scene));
    }

    #[test]
    fn detect_hdri_extensions() {
        assert_eq!(AssetType::from_extension("hdr"), Some(AssetType::Hdri));
        assert_eq!(AssetType::from_extension("exr"), Some(AssetType::Hdri));
    }

    #[test]
    fn detect_animation() {
        assert_eq!(
            AssetType::from_extension("anim"),
            Some(AssetType::Animation)
        );
    }

    #[test]
    fn detect_audio_extensions() {
        for ext in &["wav", "ogg", "mp3", "flac"] {
            assert_eq!(
                AssetType::from_extension(ext),
                Some(AssetType::Audio),
                "Failed for extension: {ext}"
            );
        }
    }

    #[test]
    fn detect_script_extensions() {
        assert_eq!(AssetType::from_extension("lua"), Some(AssetType::Script));
        assert_eq!(AssetType::from_extension("luau"), Some(AssetType::Script));
    }

    #[test]
    fn detect_font_extensions() {
        for ext in &["ttf", "otf", "woff", "woff2"] {
            assert_eq!(
                AssetType::from_extension(ext),
                Some(AssetType::Font),
                "Failed for extension: {ext}"
            );
        }
    }

    #[test]
    fn detect_shader_extensions() {
        for ext in &["wgsl", "glsl", "vert", "frag", "comp", "hlsl"] {
            assert_eq!(
                AssetType::from_extension(ext),
                Some(AssetType::Shader),
                "Failed for extension: {ext}"
            );
        }
    }

    #[test]
    fn detect_theme() {
        assert_eq!(AssetType::from_extension("theme"), Some(AssetType::Theme));
    }

    #[test]
    fn detect_prefab() {
        assert_eq!(AssetType::from_extension("prefab"), Some(AssetType::Prefab));
    }

    #[test]
    fn unknown_extension_returns_none() {
        assert_eq!(AssetType::from_extension("xyz"), None);
        assert_eq!(AssetType::from_extension(""), None);
        assert_eq!(AssetType::from_extension("doc"), None);
    }

    #[test]
    fn from_path_works() {
        assert_eq!(
            AssetType::from_path(Path::new("/assets/model.glb")),
            Some(AssetType::Mesh)
        );
    }

    #[test]
    fn from_path_no_extension() {
        assert_eq!(AssetType::from_path(Path::new("/assets/Makefile")), None);
    }

    #[test]
    fn labels_all_non_empty() {
        let types = [
            AssetType::Mesh,
            AssetType::Texture,
            AssetType::Material,
            AssetType::Scene,
            AssetType::Hdri,
            AssetType::Animation,
            AssetType::Audio,
            AssetType::Script,
            AssetType::Font,
            AssetType::Shader,
            AssetType::Theme,
            AssetType::Prefab,
        ];
        for t in &types {
            assert!(!t.label().is_empty(), "Empty label for {:?}", t);
        }
    }

    #[test]
    fn icon_chars_all_unique() {
        let types = [
            AssetType::Mesh,
            AssetType::Texture,
            AssetType::Material,
            AssetType::Scene,
            AssetType::Hdri,
            AssetType::Animation,
            AssetType::Audio,
            AssetType::Script,
            AssetType::Font,
            AssetType::Shader,
            AssetType::Theme,
            AssetType::Prefab,
        ];
        let mut chars: Vec<char> = types.iter().map(|t| t.icon_char()).collect();
        let len = chars.len();
        chars.sort();
        chars.dedup();
        assert_eq!(chars.len(), len, "Duplicate icon_char found");
    }
}
