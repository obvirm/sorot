use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use srpath::Path;
use srflatten::{flatten_path, FlattenedPath};
use srtri::{triangulate, TriMesh};

fn hash_path_and_tol(path: &Path, tolerance: f32) -> u64 {
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

pub struct GeometryCache {
    entries: HashMap<u64, (FlattenedPath, TriMesh)>,
}

impl GeometryCache {
    pub fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    pub fn get_or_compute(&mut self, path: &Path, tolerance: f32) -> (&FlattenedPath, &TriMesh) {
        let key = hash_path_and_tol(path, tolerance);
        self.entries
            .entry(key)
            .or_insert_with(|| {
                let flat = flatten_path(path, tolerance);
                let mesh = triangulate(&flat);
                (flat, mesh)
            });
        let (ref flat, ref mesh) = self.entries[&key];
        (flat, mesh)
    }

    pub fn get_mesh(&mut self, path: &Path, tolerance: f32) -> &TriMesh {
        self.get_or_compute(path, tolerance).1
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl Default for GeometryCache {
    fn default() -> Self {
        Self::new()
    }
}
