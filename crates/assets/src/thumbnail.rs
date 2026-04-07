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
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a cached thumbnail, if one exists.
    pub fn get(&self, id: AssetId) -> Option<&ThumbnailData> {
        self.cache.get(&id)
    }

    /// Inserts a thumbnail into the cache.
    pub fn insert(&mut self, id: AssetId, data: ThumbnailData) {
        self.cache.insert(id, data);
    }

    /// Removes a thumbnail from the cache.
    pub fn remove(&mut self, id: AssetId) {
        self.cache.remove(&id);
    }

    /// Returns the number of cached thumbnails.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clears all cached thumbnails.
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
}
