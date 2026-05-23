use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use crate::flatten::{flatten_path, FlattenedPath};
use crate::path::Path;

pub struct PathCache {
    flattened: HashMap<u64, FlattenedPath>,
}

impl PathCache {
    pub fn new() -> Self {
        Self {
            flattened: HashMap::new(),
        }
    }

    pub fn get_or_flatten(&mut self, path: &Path, tolerance: f32) -> &FlattenedPath {
        let key = hash_path(path);
        self.flattened
            .entry(key)
            .or_insert_with(|| flatten_path(path, tolerance))
    }

    pub fn clear(&mut self) {
        self.flattened.clear();
    }

    pub fn len(&self) -> usize {
        self.flattened.len()
    }

    pub fn is_empty(&self) -> bool {
        self.flattened.is_empty()
    }
}

impl Default for PathCache {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_path(path: &Path) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for verb in path.verbs() {
        (*verb as u8).hash(&mut hasher);
    }
    for point in path.points() {
        point.x.to_bits().hash(&mut hasher);
        point.y.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::Path;
    use sorot_core::math::Vec2;

    #[test]
    fn test_cache_hit() {
        let mut cache = PathCache::new();
        let path = Path::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));

        let flat1_ptr = {
            let flat = cache.get_or_flatten(&path, 1.0);
            flat as *const FlattenedPath
        };
        let flat2_ptr = {
            let flat = cache.get_or_flatten(&path, 1.0);
            flat as *const FlattenedPath
        };

        assert_eq!(flat1_ptr, flat2_ptr);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = PathCache::new();
        let p1 = Path::circle(Vec2::new(50.0, 50.0), 30.0);
        let p2 = Path::circle(Vec2::new(100.0, 100.0), 30.0);

        cache.get_or_flatten(&p1, 0.5);
        cache.get_or_flatten(&p2, 0.5);
        assert_eq!(cache.len(), 2);
    }
}
