use super::*;

    #[test]
    fn test_edge_new_top_to_bottom() {
        let e = Edge::new(10.0, 0.0, 10.0, 10.0, 1).unwrap();
        assert_eq!(e.y0, 0);
        assert_eq!(e.y1, 10);
    }

    #[test]
    fn test_edge_new_horizontal_skipped() {
        let e = Edge::new(0.0, 5.0, 10.0, 5.0, 1);
        assert!(e.is_none());
    }

    #[test]
    fn test_edge_new_bottom_to_top_flipped() {
        let e = Edge::new(5.0, 10.0, 5.0, 0.0, 1).unwrap();
        assert!(e.y0 <= e.y1);
    }

    #[test]
    fn test_edge_advance() {
        let mut e = Edge::new(0.0, 0.0, 10.0, 10.0, 1).unwrap();
        let x_before = e.x;
        e.advance();
        assert!((e.x - x_before - 1.0).abs() < 1e-6);
        assert_eq!(e.y0, 1);
    }

    #[test]
    fn test_tile_grid_creation() {
        let grid = TileGrid::new(100, 100, 16);
        assert_eq!(grid.cols, 7);
        assert_eq!(grid.rows, 7);
        assert_eq!(grid.tiles.len(), 49);
    }

    #[test]
    fn test_bin_edges() {
        let mut grid = TileGrid::new(32, 32, 16);
        let e = Edge::new(5.0, 5.0, 25.0, 25.0, 1).unwrap();
        grid.bin_edges(&[e]);
        let total_edges: usize = grid.tiles.iter().map(|t| t.edges.len()).sum();
        assert!(total_edges > 0);
    }
