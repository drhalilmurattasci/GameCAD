//! Thumbnail generation and caching for the asset browser.

use std::collections::HashMap;
use std::path::Path;

use forge_core::id::AssetId;
use tracing::{debug, warn};

/// Raw pixel data for a thumbnail image.
#[derive(Debug, Clone)]
pub struct ThumbnailData {
    /// Width of the thumbnail in pixels.
    pub width: u32,
    /// Height of the thumbnail in pixels.
    pub height: u32,
    /// RGBA pixel data, row-major, 4 bytes per pixel.
    pub pixels: Vec<u8>,
}

/// In-memory cache of generated thumbnails, keyed by asset ID.
#[derive(Debug, Default)]
pub struct ThumbnailCache {
    cache: HashMap<AssetId, ThumbnailData>,
}

/// Default thumbnail size (square).
const THUMB_SIZE: u32 = 128;

impl ThumbnailCache {
    /// Creates an empty cache.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a cached thumbnail, if one exists.
    #[inline]
    pub fn get(&self, id: AssetId) -> Option<&ThumbnailData> {
        self.cache.get(&id)
    }

    /// Inserts a thumbnail into the cache.
    #[inline]
    pub fn insert(&mut self, id: AssetId, data: ThumbnailData) {
        self.cache.insert(id, data);
    }

    /// Removes a thumbnail from the cache.
    #[inline]
    pub fn remove(&mut self, id: AssetId) {
        self.cache.remove(&id);
    }

    /// Returns the number of cached thumbnails.
    #[inline]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns `true` if the cache is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clears all cached thumbnails.
    #[inline]
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

/// Generates a thumbnail for a texture file by loading and down-scaling it.
///
/// Returns `None` if the image cannot be loaded.
pub fn generate_texture_thumbnail(path: &Path) -> Option<ThumbnailData> {
    let img = match image::open(path) {
        Ok(img) => img,
        Err(e) => {
            warn!("Failed to open image for thumbnail: {}", e);
            return None;
        }
    };

    let thumb = img.resize_exact(
        THUMB_SIZE,
        THUMB_SIZE,
        image::imageops::FilterType::Triangle,
    );
    let rgba = thumb.to_rgba8();

    debug!(
        "Generated texture thumbnail for {} ({}x{})",
        path.display(),
        THUMB_SIZE,
        THUMB_SIZE
    );

    Some(ThumbnailData {
        width: THUMB_SIZE,
        height: THUMB_SIZE,
        pixels: rgba.into_raw(),
    })
}

/// Generates a placeholder colored rectangle thumbnail for a mesh asset.
///
/// The color is derived from a simple hash of the `name` string so that the
/// same asset always gets the same placeholder color.
pub fn generate_mesh_placeholder(name: &str) -> ThumbnailData {
    // Simple hash to pick a color.
    let hash = name
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));

    let r = ((hash >> 16) & 0xFF) as u8 | 0x40;
    let g = ((hash >> 8) & 0xFF) as u8 | 0x40;
    let b = (hash & 0xFF) as u8 | 0x40;

    let pixel_count = (THUMB_SIZE * THUMB_SIZE) as usize;
    let mut pixels = Vec::with_capacity(pixel_count * 4);

    for row in 0..THUMB_SIZE {
        for col in 0..THUMB_SIZE {
            // Draw a simple border to make it look like a card.
            let is_border = row < 2 || col < 2 || row >= THUMB_SIZE - 2 || col >= THUMB_SIZE - 2;
            if is_border {
                pixels.extend_from_slice(&[60, 60, 60, 255]);
            } else {
                pixels.extend_from_slice(&[r, g, b, 255]);
            }
        }
    }

    ThumbnailData {
        width: THUMB_SIZE,
        height: THUMB_SIZE,
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_placeholder_deterministic() {
        let a = generate_mesh_placeholder("Cube");
        let b = generate_mesh_placeholder("Cube");
        assert_eq!(a.pixels, b.pixels);
    }

    #[test]
    fn mesh_placeholder_correct_size() {
        let thumb = generate_mesh_placeholder("Sphere");
        assert_eq!(thumb.width, THUMB_SIZE);
        assert_eq!(thumb.height, THUMB_SIZE);
        assert_eq!(thumb.pixels.len(), (THUMB_SIZE * THUMB_SIZE * 4) as usize);
    }

    #[test]
    fn cache_operations() {
        let mut cache = ThumbnailCache::new();
        assert!(cache.is_empty());

        let id = AssetId::new();
        let data = generate_mesh_placeholder("test");
        cache.insert(id, data);
        assert_eq!(cache.len(), 1);
        assert!(cache.get(id).is_some());

        cache.remove(id);
        assert!(cache.is_empty());
    }

    #[test]
    fn mesh_placeholder_empty_name() {
        let thumb = generate_mesh_placeholder("");
        assert_eq!(thumb.width, THUMB_SIZE);
        assert_eq!(thumb.height, THUMB_SIZE);
        assert_eq!(thumb.pixels.len(), (THUMB_SIZE * THUMB_SIZE * 4) as usize);
    }

    #[test]
    fn mesh_placeholder_long_name() {
        let long_name = "A".repeat(10_000);
        let thumb = generate_mesh_placeholder(&long_name);
        assert_eq!(thumb.pixels.len(), (THUMB_SIZE * THUMB_SIZE * 4) as usize);
    }

    #[test]
    fn different_names_produce_different_colors() {
        let a = generate_mesh_placeholder("Alpha");
        let b = generate_mesh_placeholder("Beta");
        // The interior pixels (not border) should differ.
        // Check pixel at (64, 64) -> index = (64 * 128 + 64) * 4
        let idx = (64 * THUMB_SIZE as usize + 64) * 4;
        assert_ne!(&a.pixels[idx..idx + 3], &b.pixels[idx..idx + 3]);
    }

    #[test]
    fn mesh_placeholder_has_border() {
        let thumb = generate_mesh_placeholder("Test");
        // Top-left corner should be the border color (60, 60, 60, 255).
        assert_eq!(&thumb.pixels[0..4], &[60, 60, 60, 255]);
        // Bottom-right corner too.
        let last_idx = ((THUMB_SIZE * THUMB_SIZE - 1) * 4) as usize;
        assert_eq!(&thumb.pixels[last_idx..last_idx + 4], &[60, 60, 60, 255]);
    }

    #[test]
    fn cache_clear() {
        let mut cache = ThumbnailCache::new();
        cache.insert(AssetId::new(), generate_mesh_placeholder("a"));
        cache.insert(AssetId::new(), generate_mesh_placeholder("b"));
        assert_eq!(cache.len(), 2);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn cache_insert_overwrites() {
        let mut cache = ThumbnailCache::new();
        let id = AssetId::new();
        cache.insert(id, generate_mesh_placeholder("v1"));
        cache.insert(id, generate_mesh_placeholder("v2"));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn cache_remove_nonexistent_is_noop() {
        let mut cache = ThumbnailCache::new();
        cache.remove(AssetId::new()); // Should not panic.
        assert!(cache.is_empty());
    }

    #[test]
    fn texture_thumbnail_nonexistent_returns_none() {
        let result = generate_texture_thumbnail(std::path::Path::new("/nonexistent/path.png"));
        assert!(result.is_none());
    }

    #[test]
    fn texture_thumbnail_invalid_file_returns_none() {
        let dir = std::env::temp_dir().join("forge_thumb_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("not_an_image.png");
        std::fs::write(&path, b"this is not a png").unwrap();
        let result = generate_texture_thumbnail(&path);
        assert!(result.is_none());
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }
}
