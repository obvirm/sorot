use super::*;

    #[test]
    fn test_surface_new() {
        let s = Surface::new(100, 50);
        assert_eq!(s.pixels.len(), 20000);
    }

    #[test]
    fn test_fill() {
        let mut s = Surface::new(10, 10);
        s.fill(Color::from_rgba(1.0, 0.0, 0.0, 1.0));
        let pixel = s.get_pixel(5, 5);
        assert!((pixel.r - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_composite_src_over() {
        let mut dst = Surface::new(10, 10);
        dst.fill(Color::from_rgba(0.0, 0.0, 1.0, 1.0));

        let mut src = Surface::new(5, 5);
        src.fill(Color::from_rgba(1.0, 0.0, 0.0, 0.5));

        dst.composite(&src, 0, 0, BlendMode::SrcOver);
        let pixel = dst.get_pixel(2, 2);
        assert!(pixel.r > 0.0);
        assert!(pixel.b > 0.0);
    }
