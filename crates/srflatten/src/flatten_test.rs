use super::*;
    use srvec2::Vec2;

    #[test]
    fn test_flatten_line() {
        let mut p = Path::new();
        p.move_to(Vec2::new(0.0, 0.0));
        p.line_to(Vec2::new(100.0, 0.0));
        let f = flatten_path(&p, 1.0);
        assert_eq!(f.verbs.len(), 2); // M, L
    }

    #[test]
    fn test_flatten_rect() {
        let p = Path::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0));
        let f = flatten_path(&p, 1.0);
        assert_eq!(f.verbs.len(), 6); // M, L, L, L, L, Z
    }

    #[test]
    fn test_flatten_circle() {
        let p = Path::circle(Vec2::new(50.0, 50.0), 40.0);
        let f = flatten_path(&p, 0.5);
        assert!(f.verbs.len() > 4);
    }

    #[test]
    fn test_flatten_quad() {
        let mut p = Path::new();
        p.move_to(Vec2::new(0.0, 0.0));
        p.quad_to(Vec2::new(50.0, 100.0), Vec2::new(100.0, 0.0));
        let f = flatten_path(&p, 0.5);
        assert!(f.verbs.len() >= 2); // M + at least one L
    }

    #[test]
    fn test_is_flat_enough_straight() {
        let c = Cubic::new(
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(2.0, 0.0),
            Vec2::new(3.0, 0.0),
        );
        assert!(is_flat_enough(&c, 1.0));
    }
