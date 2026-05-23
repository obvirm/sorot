use std::hash::{Hash, Hasher};

use crate::flatten::{flatten_path, FlattenedPath};
use crate::path::Path;

pub struct PathCache {
    entries: std::collections::HashMap<u64, FlattenedPath>,
}

impl PathCache {
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    pub fn get_or_flatten(&mut self, path: &Path, tolerance: f32) -> &FlattenedPath {
        let key = make_key(path, tolerance);
        self.entries
            .entry(key)
            .or_insert_with(|| flatten_path(path, tolerance))
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Default for PathCache {
    fn default() -> Self {
        Self::new()
    }
}

fn make_key(path: &Path, tolerance: f32) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for v in path.verbs() {
        (*v as u8).hash(&mut h);
    }
    for p in path.points() {
        p.x.to_bits().hash(&mut h);
        p.y.to_bits().hash(&mut h);
    }
    tolerance.to_bits().hash(&mut h);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::path::Path;
    use srcore::math::Vec2;

    #[test]
    fn test_cache_hit() {
        let mut c = PathCache::new();
        let p = Path::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let a = c.get_or_flatten(&p, 0.5) as *const _;
        let b = c.get_or_flatten(&p, 0.5) as *const _;
        assert_eq!(a, b);
    }

    #[test]
    fn test_different_tolerance_miss() {
        let mut c = PathCache::new();
        let p = Path::circle(Vec2::new(50.0, 50.0), 30.0);
        c.get_or_flatten(&p, 0.5);
        c.get_or_flatten(&p, 1.0);
        assert_eq!(c.len(), 2);
    }
}
