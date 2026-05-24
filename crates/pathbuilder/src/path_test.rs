use super::*;

    #[test]
    fn test_path_builder() {
        let mut p = Path::new();
        p.move_to(Vec2::new(0.0, 0.0));
        p.line_to(Vec2::new(10.0, 0.0));
        p.line_to(Vec2::new(10.0, 10.0));
        p.close();
        assert_eq!(p.verb_count(), 4);
        assert_eq!(p.point_count(), 3);
    }

    #[test]
    fn test_path_iter() {
        let mut p = Path::new();
        p.move_to(Vec2::new(0.0, 0.0));
        p.line_to(Vec2::new(10.0, 5.0));
        p.quad_to(Vec2::new(5.0, 10.0), Vec2::new(0.0, 5.0));
        p.close();

        let segments: Vec<_> = p.iter().collect();
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].verb, PathVerb::MoveTo);
        assert_eq!(segments[1].verb, PathVerb::LineTo);
        assert_eq!(segments[2].verb, PathVerb::QuadTo);
        assert_eq!(segments[3].verb, PathVerb::Close);
    }

    #[test]
    fn test_rect() {
        let r = Path::rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 50.0));
        assert_eq!(r.verb_count(), 5); // M, L, L, L, Z
    }

    #[test]
    fn test_oval() {
        let o = Path::oval(Vec2::new(50.0, 50.0), 30.0, 20.0);
        assert!(o.verb_count() > 0);
    }

    #[test]
    fn test_rounded_rect() {
        let r = Path::rounded_rect(Vec2::new(0.0, 0.0), Vec2::new(100.0, 100.0), 10.0, 10.0);
        assert_eq!(r.verb_count(), 10); // M, L, C, L, C, L, C, L, C, Z
    }
