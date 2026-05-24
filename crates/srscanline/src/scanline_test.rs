use super::*;
    use srvec2::Vec2;

    #[test]
    fn test_rasterize_triangle() {
        let mut path = FlattenedPath::new();
        path.verbs.push(FlattenVerb::MoveTo);
        path.points.push(Vec2::new(50.0, 10.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(90.0, 90.0));
        path.verbs.push(FlattenVerb::LineTo);
        path.points.push(Vec2::new(10.0, 90.0));
        path.verbs.push(FlattenVerb::Close);

        let width = 100;
        let height = 100;
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let color = Color::from_rgba(1.0, 0.0, 0.0, 1.0);

        rasterize_path(&path, color, &mut buffer, width, height, false);

        let center_px = ((50 * width as usize + 50) * 4) as usize;
        assert!(buffer[center_px + 3] > 0, "center pixel should be filled");

        let corner_px = ((5 * width as usize + 5) * 4) as usize;
        assert_eq!(buffer[corner_px + 3], 0, "corner pixel should be empty");
    }

    #[test]
    fn test_rasterize_square_even_odd() {
        let mut outer = FlattenedPath::new();
        outer.verbs.push(FlattenVerb::MoveTo);
        outer.points.push(Vec2::new(0.0, 0.0));
        outer.verbs.push(FlattenVerb::LineTo);
        outer.points.push(Vec2::new(100.0, 0.0));
        outer.verbs.push(FlattenVerb::LineTo);
        outer.points.push(Vec2::new(100.0, 100.0));
        outer.verbs.push(FlattenVerb::LineTo);
        outer.points.push(Vec2::new(0.0, 100.0));
        outer.verbs.push(FlattenVerb::Close);

        let width = 100;
        let height = 100;
        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let color = Color::from_rgba(0.0, 0.0, 1.0, 1.0);

        rasterize_path(&outer, color, &mut buffer, width, height, true);

        let center = ((50 * width as usize + 50) * 4) as usize;
        assert!(buffer[center + 3] > 0);
    }
