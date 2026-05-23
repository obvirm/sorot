use super::*;
use super::super::path::Path;
use crate::math::Vec2;

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
