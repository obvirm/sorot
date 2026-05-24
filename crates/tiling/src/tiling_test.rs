use super::*;
    use vector::Vec2;

    #[test]
    fn test_classify_tiles() {
        let grid = TileGrid::new(64, 64, 16);
        let viewport = Rect::new(Vec2::new(0.0, 0.0), Vec2::new(32.0, 32.0));

        let (v, f, b) = TileScheduler::classify_tiles(&grid, viewport);
        assert!(!v.is_empty());
        assert!(v.len() + f.len() + b.len() == grid.tiles.len());
    }
